//! Domain Orchestration: checkpoint capture and deterministic replay from an
//! interrupted run.
//!
//! ADR-0108 (Checkpoint & Replay); docs/ddd/12-operations-failure.md.
//! `replay` must reach the same output digest as an uninterrupted run; any divergence
//! is a correctness failure, not an acceptable approximation
//! (docs/implementation-contract.md).
//!
//! **Documented deviation**: docs/implementation-contract.md's planning-table
//! signature is `replay(&Checkpoint, remaining: WorkloadTrace) -> SimulationRun`. In
//! this implementation every initial arrival is injected into `TenantQueues` at
//! capture time (ADR-0102), so there is no second wave of *new* arrivals left to
//! inject at replay time — `remaining` is kept as a parameter and used as a
//! consistency check (its seed must match the checkpoint's) rather than as additional
//! input data, which is what the signature is shown for.

use jitter_outcome::OutcomeEvent;
use jitter_scenario::ValidatedScenario;
use jitter_scheduler::TenantQueues;
use jitter_simulation::{AdmissionConfig, SimulationRun};
use jitter_workload::WorkloadTrace;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub tick: u32,
    queue_state: TenantQueues,
    budget_state: jitter_budget::Budget,
    events_so_far: Vec<OutcomeEvent>,
    scenario_digest: String,
    original_requests: u32,
    tenant_original_requests: Vec<u32>,
    tenants: u32,
    max_events: u64,
    config: AdmissionConfig,
}

/// Runs `scenario` under `config` up to and including `interrupt_after_tick`, then
/// stops and snapshots every piece of mutable state needed to resume exactly where it
/// left off. This is what a real interruption signal (`jitter-adapter-ruflo`'s
/// `HarnessSink`, ADR-0114) triggers in production.
pub fn capture(
    scenario: &ValidatedScenario,
    seed: u64,
    config: &AdmissionConfig,
    interrupt_after_tick: u32,
) -> Checkpoint {
    let (mut queues, mut budget) = jitter_simulation::initial_state(scenario, seed, config);
    let policy = jitter_simulation::policy_for(config);
    let max_events = scenario.scenario().max_events;
    let events = jitter_simulation::drive(
        &mut queues,
        &mut budget,
        policy.as_ref(),
        max_events,
        Some(interrupt_after_tick),
    );
    Checkpoint {
        tick: interrupt_after_tick,
        queue_state: queues,
        budget_state: budget,
        events_so_far: events,
        scenario_digest: scenario.digest().to_string(),
        original_requests: scenario.requests(),
        tenant_original_requests: jitter_simulation::tenant_original_requests(scenario, seed),
        tenants: scenario.tenants(),
        max_events,
        config: config.clone(),
    }
}

/// Resumes a `Checkpoint` to a final [`SimulationRun`], reconstructing the exact
/// policy the interrupted run used and continuing the tick loop from the saved queue
/// and budget state.
pub fn replay(checkpoint: &Checkpoint, remaining: WorkloadTrace) -> SimulationRun {
    debug_assert_eq!(
        remaining.seed(),
        checkpoint.queue_state.seed,
        "replay seed must match the interrupted run's seed"
    );
    let mut queues = checkpoint.queue_state.clone();
    let mut budget = checkpoint.budget_state;
    let policy = jitter_simulation::policy_for(&checkpoint.config);
    let resumed_events = jitter_simulation::drive(
        &mut queues,
        &mut budget,
        policy.as_ref(),
        checkpoint.max_events,
        None,
    );
    let mut events = checkpoint.events_so_far.clone();
    events.extend(resumed_events);
    jitter_simulation::finalize(
        checkpoint.scenario_digest.clone(),
        checkpoint.original_requests,
        checkpoint.tenant_original_requests.clone(),
        checkpoint.tenants,
        checkpoint.config.strategy,
        &events,
    )
}

/// SHA-256 digest of a run's canonical JSON serialization — the comparable artifact
/// for "interrupted-and-replayed digest equals uninterrupted digest"
/// (docs/architecture.md §7).
pub fn run_digest(run: &SimulationRun) -> String {
    let json = serde_json::to_string(run).expect("SimulationRun always serializes");
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jitter_outcome::AdmissionStrategy;
    use jitter_scenario::{FailureClass, Scenario, validate};

    fn scenario() -> ValidatedScenario {
        validate(Scenario {
            version: 1,
            requests: 500,
            tenants: 20,
            capacity_per_tick: 64,
            outage_ticks: 12,
            horizon_ticks: 200,
            seed: 424242,
            failure: FailureClass::Throttling,
            tenant_weights: Some((0..20).map(|i| if i < 3 { 20 } else { 1 }).collect()),
            max_events: 200_000,
        })
        .unwrap()
    }

    fn config() -> AdmissionConfig {
        AdmissionConfig {
            strategy: AdmissionStrategy::WeightedTenantFairBudget,
            max_attempts: 6,
            base_delay: 1,
            cap_delay: 32,
            shared_budget: 1500,
            refill_per_tick: 18,
            failure_cost: 1,
        }
    }

    #[test]
    fn interrupted_replay_matches_uninterrupted_digest_at_tick_six() {
        let uninterrupted = jitter_simulation::run(&scenario(), 424242, &config());
        let uninterrupted_digest = run_digest(&uninterrupted);

        let checkpoint = capture(&scenario(), 424242, &config(), 6);
        let remaining = jitter_workload::generate(&scenario(), 424242);
        let replayed = replay(&checkpoint, remaining);
        let replayed_digest = run_digest(&replayed);

        assert_eq!(uninterrupted_digest, replayed_digest);
        assert_eq!(uninterrupted, replayed);
    }

    #[test]
    fn checkpoint_captures_partial_progress() {
        let checkpoint = capture(&scenario(), 424242, &config(), 6);
        assert_eq!(checkpoint.tick, 6);
        assert!(!checkpoint.events_so_far.is_empty());
    }

    #[test]
    fn replay_is_deterministic_across_repeated_calls() {
        let checkpoint = capture(&scenario(), 424242, &config(), 6);
        let remaining_a = jitter_workload::generate(&scenario(), 424242);
        let remaining_b = jitter_workload::generate(&scenario(), 424242);
        let a = replay(&checkpoint, remaining_a);
        let b = replay(&checkpoint, remaining_b);
        assert_eq!(a, b);
    }

    #[test]
    fn interruption_deep_inside_outage_window_still_replays_identically() {
        // tick 9 is inside the 12-tick outage window and mid-quantum for the
        // weighted-fair rotation — the hardest point to resume correctly.
        let uninterrupted = jitter_simulation::run(&scenario(), 424242, &config());
        let checkpoint = capture(&scenario(), 424242, &config(), 9);
        let remaining = jitter_workload::generate(&scenario(), 424242);
        let replayed = replay(&checkpoint, remaining);
        assert_eq!(uninterrupted, replayed);
    }
}
