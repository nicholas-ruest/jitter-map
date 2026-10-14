//! Domain Orchestration: the pure deterministic simulation kernel composing every
//! Domain Core crate (`jitter-scenario`, `jitter-workload`, `jitter-policy`,
//! `jitter-budget`, `jitter-scheduler`, `jitter-outcome`) into one run.
//!
//! ADR-0107 (Pure Simulation Kernel); docs/ddd/05-simulation-engine.md. Zero I/O,
//! clock, or thread dependency — this crate's `Cargo.toml` names only Domain Core
//! crates plus `serde`, which is what the structural checker (docs/architecture.md §8)
//! verifies.
//!
//! **Documented deviation from docs/implementation-contract.md's planning-table
//! signature** `fn run(&ValidatedScenario, seed: u64) -> SimulationRun`: a bare
//! scenario and seed cannot determine which of the six admission strategies to run
//! or what budget it has, because strategy and budget are adapter-sourced inputs
//! (ADR-0005/ADR-0111), not scenario fields. `run` therefore also takes an
//! [`AdmissionConfig`] naming the strategy and its resolved budget parameters. This
//! keeps `run` infallible and clock/IO-free — the two load-bearing parts of the
//! contract — while making it possible to run more than one strategy at all.
//!
//! **Documented deviation on event-ceiling handling**: the base `3db7fd2` monolith
//! returned `DomainError::EventLimit` once `attempts >= scenario.max_events`. Because
//! this crate's `run` is contractually infallible, reaching the ceiling here instead
//! stops admitting further ticks deterministically (the run is truncated, not
//! errored). `fleet-retry-storm-22` and every test fixture in this workspace stay far
//! under their configured `max_events`, so this divergence is never exercised by the
//! acceptance fixture; it exists only as a documented safety bound.

use jitter_budget::Budget;
use jitter_outcome::{AdmissionStrategy, OutcomeEvent, OutcomeSummary};
use jitter_policy::{DemandWeighted, ExponentialJitter, RetryPolicy};
use jitter_scenario::ValidatedScenario;
use jitter_scheduler::{TenantQueues, Tick, schedule_tick};
use serde::{Deserialize, Serialize};

/// The admission strategy plus the resolved budget/delay parameters it runs with.
/// Sourced at a composition root from `RetryConfigSource` (ADR-0111).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdmissionConfig {
    pub strategy: AdmissionStrategy,
    pub max_attempts: u32,
    pub base_delay: u32,
    pub cap_delay: u32,
    pub shared_budget: u32,
    pub refill_per_tick: u32,
    pub failure_cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SimulationRun {
    pub scenario_digest: String,
    pub original_requests: u32,
    pub tenant_original_requests: Vec<u32>,
    pub summary: OutcomeSummary,
}

impl SimulationRun {
    pub fn retry_amplification_factor(&self) -> f64 {
        jitter_outcome::retry_amplification_factor(self.summary.attempts, self.original_requests)
    }

    pub fn p95_ticks(&self) -> u32 {
        jitter_outcome::p95_ticks(&self.summary.completed_at)
    }

    pub fn tenant_success_fairness(&self) -> Option<f64> {
        jitter_outcome::tenant_success_fairness(
            &self.tenant_original_requests,
            &self.summary.tenant_successes,
        )
    }
}

/// Builds the stateless delay policy for `config`. Boxed because `jitter-recovery`
/// must be able to rebuild the identical policy later from a checkpoint's stored
/// `AdmissionConfig`, long after the original `&dyn RetryPolicy` borrow has ended.
pub fn policy_for(config: &AdmissionConfig) -> Box<dyn RetryPolicy> {
    let jitter = ExponentialJitter {
        base_delay: config.base_delay,
        cap_delay: config.cap_delay,
        max_attempts: config.max_attempts,
    };
    if config.strategy.demand_weighted() {
        Box::new(DemandWeighted(jitter))
    } else {
        Box::new(jitter)
    }
}

/// Computes each tenant's share of the initial workload trace, independent of
/// admission strategy — needed by both a fresh run and a replay to report fairness.
pub fn tenant_original_requests(scenario: &ValidatedScenario, seed: u64) -> Vec<u32> {
    let trace = jitter_workload::generate(scenario, seed);
    let mut counts = vec![0u32; scenario.tenants() as usize];
    for request in trace.requests() {
        counts[request.tenant_id as usize] += 1;
    }
    counts
}

/// Builds the initial `(TenantQueues, Budget)` state for `scenario` under `config`,
/// with the tick-0 workload trace already seeded into the queues. This is the exact
/// state a fresh [`run`] starts from and the state `jitter-recovery` must be able to
/// reconstruct identically before fast-forwarding to a checkpoint.
pub fn initial_state(
    scenario: &ValidatedScenario,
    seed: u64,
    config: &AdmissionConfig,
) -> (TenantQueues, Budget) {
    let trace = jitter_workload::generate(scenario, seed);
    let raw = scenario.scenario();
    let budget = Budget::new(config.shared_budget, config.refill_per_tick);
    let mut queues = TenantQueues::new(
        scenario.tenants(),
        scenario.tenant_weights().map(<[u32]>::to_vec),
        config.strategy.fair_ordering(),
        config.strategy.demand_weighted(),
        config.strategy.budget_gated(),
        config.strategy.deferral_enabled(),
        raw.capacity_per_tick,
        raw.outage_ticks,
        raw.horizon_ticks,
        config.failure_cost,
        trace.seed(),
        raw.failure,
    );
    for request in trace {
        queues.schedule_at(
            request.tick,
            request.tenant_id,
            request.request_id,
            request.attempt,
        );
    }
    (queues, budget)
}

/// Drains `queues` tick by tick through `schedule_tick`, mutating `queues` and
/// `budget` in place and returning every emitted event. If `stop_after_tick` is
/// `Some(t)`, draining halts once every pending tick `<= t` has been processed,
/// leaving any events scheduled beyond `t` still pending in `queues` — exactly the
/// mid-run interruption point `jitter-recovery` checkpoints from.
pub fn drive(
    queues: &mut TenantQueues,
    budget: &mut Budget,
    policy: &dyn RetryPolicy,
    max_events: u64,
    stop_after_tick: Option<u32>,
) -> Vec<OutcomeEvent> {
    let mut events = Vec::new();
    let mut counted_attempts: u64 = 0;
    while let Some(tick) = queues.next_tick() {
        if let Some(stop) = stop_after_tick {
            if tick > stop {
                break;
            }
        }
        if counted_attempts >= max_events {
            break;
        }
        let result = schedule_tick(Tick(tick), queues, budget, policy);
        for event in result.events {
            if !matches!(event, OutcomeEvent::HorizonExceeded { .. }) {
                counted_attempts += 1;
            }
            events.push(event);
        }
    }
    events
}

/// Aggregates a completed (or resumed-to-completion) event log into the final
/// [`SimulationRun`].
pub fn finalize(
    scenario_digest: String,
    original_requests: u32,
    tenant_original_requests: Vec<u32>,
    tenants: u32,
    strategy: AdmissionStrategy,
    events: &[OutcomeEvent],
) -> SimulationRun {
    let summary =
        jitter_outcome::with_strategy(jitter_outcome::summarize(events, tenants), strategy);
    SimulationRun {
        scenario_digest,
        original_requests,
        tenant_original_requests,
        summary,
    }
}

/// Runs `fleet-retry-storm-22`-shaped scenarios (or any `ValidatedScenario`) under one
/// admission strategy to a deterministic, final [`SimulationRun`]. Infallible given
/// valid input; zero I/O, clock, or thread dependency.
pub fn run(scenario: &ValidatedScenario, seed: u64, config: &AdmissionConfig) -> SimulationRun {
    let (mut queues, mut budget) = initial_state(scenario, seed, config);
    let policy = policy_for(config);
    let events = drive(
        &mut queues,
        &mut budget,
        policy.as_ref(),
        scenario.scenario().max_events,
        None,
    );
    finalize(
        scenario.digest().to_string(),
        scenario.requests(),
        tenant_original_requests(scenario, seed),
        scenario.tenants(),
        config.strategy,
        &events,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use jitter_scenario::{FailureClass, Scenario, validate};

    fn scenario() -> ValidatedScenario {
        validate(Scenario {
            version: 1,
            requests: 100,
            tenants: 4,
            capacity_per_tick: 10,
            outage_ticks: 3,
            horizon_ticks: 100,
            seed: 7,
            failure: FailureClass::Throttling,
            tenant_weights: None,
            max_events: 10_000,
        })
        .unwrap()
    }

    fn config(strategy: AdmissionStrategy) -> AdmissionConfig {
        AdmissionConfig {
            strategy,
            max_attempts: 5,
            base_delay: 1,
            cap_delay: 16,
            shared_budget: 40,
            refill_per_tick: 2,
            failure_cost: 3,
        }
    }

    #[test]
    fn run_is_deterministic_across_repeated_calls() {
        let a = run(&scenario(), 7, &config(AdmissionStrategy::AdaptiveBudget));
        let b = run(&scenario(), 7, &config(AdmissionStrategy::AdaptiveBudget));
        assert_eq!(a, b);
    }

    #[test]
    fn budget_limits_amplification_versus_unbounded_local_jitter() {
        let bounded = run(&scenario(), 7, &config(AdmissionStrategy::AdaptiveBudget));
        let mut unbounded_cfg = config(AdmissionStrategy::LocalJitter);
        unbounded_cfg.shared_budget = u32::MAX;
        unbounded_cfg.refill_per_tick = 0;
        let unbounded = run(&scenario(), 7, &unbounded_cfg);
        assert!(bounded.summary.attempts < unbounded.summary.attempts);
        assert!(bounded.summary.denied_retries > 0);
    }

    #[test]
    fn deferral_recovers_denied_budget_versus_adaptive_drop() {
        let dropped = run(&scenario(), 7, &config(AdmissionStrategy::AdaptiveBudget));
        let deferred = run(&scenario(), 7, &config(AdmissionStrategy::AdaptiveDeferral));
        assert!(deferred.summary.deferred_retries > 0);
        assert!(deferred.summary.successes > dropped.summary.successes);
    }

    #[test]
    fn weighted_tenant_fair_improves_jain_fairness_on_skewed_demand() {
        let skewed = validate(Scenario {
            version: 1,
            requests: 150,
            tenants: 3,
            capacity_per_tick: 5,
            outage_ticks: 4,
            horizon_ticks: 100,
            seed: 20261009,
            failure: FailureClass::Throttling,
            tenant_weights: Some(vec![8, 1, 1]),
            max_events: 10_000,
        })
        .unwrap();
        let mut cfg = config(AdmissionStrategy::TenantFairBudget);
        cfg.shared_budget = 120;
        cfg.refill_per_tick = 4;
        cfg.failure_cost = 1;
        let equal_share = run(&skewed, 1, &cfg);
        cfg.strategy = AdmissionStrategy::WeightedTenantFairBudget;
        let weighted = run(&skewed, 1, &cfg);
        assert!(weighted.summary.successes >= equal_share.summary.successes);
        assert!(
            weighted.tenant_success_fairness().unwrap()
                > equal_share.tenant_success_fairness().unwrap()
        );
    }

    /// Golden-value regression guard for row 3's bit-for-bit acceptance criterion.
    ///
    /// These five numbers per strategy (`attempts`, `successes`, `terminal_failures`,
    /// `denied_retries`, `deferred_retries`) were captured by running this exact
    /// scenario+policy pairing through both this crate's `run` *and* the base
    /// `3db7fd2` monolith's `jitter_domain::simulate` side by side — including a full
    /// comparison of `tenant_successes`, `tenant_denied_retries`, and `completed_at`,
    /// which all matched exactly — before `jitter-domain` was deleted as this
    /// migration's replacement landed (docs/architecture.md §6: "delete the old
    /// in-monolith implementation of each piece as its replacement lands"). This test
    /// freezes that verified-equal result so the comparison survives the deletion.
    #[test]
    fn matches_base_monolith_golden_values_across_every_strategy() {
        let scenario = validate(Scenario {
            version: 1,
            requests: 400,
            tenants: 8,
            capacity_per_tick: 25,
            outage_ticks: 6,
            horizon_ticks: 200,
            seed: 20261004,
            failure: FailureClass::Throttling,
            tenant_weights: None,
            max_events: 100_000,
        })
        .unwrap();

        // (strategy, max_attempts, attempts, successes, terminal_failures, denied_retries, deferred_retries)
        let golden = [
            (
                AdmissionStrategy::LocalJitter,
                6,
                2175u64,
                364u32,
                36u32,
                0u64,
                0u64,
            ),
            (AdmissionStrategy::AwsStandard, 3, 1200, 0, 400, 0, 0),
            (AdmissionStrategy::AdaptiveBudget, 6, 2122, 332, 68, 38, 0),
            (AdmissionStrategy::AdaptiveDeferral, 6, 2177, 329, 71, 0, 55),
            (AdmissionStrategy::TenantFairBudget, 6, 2115, 336, 64, 38, 0),
            (
                AdmissionStrategy::WeightedTenantFairBudget,
                6,
                2115,
                336,
                64,
                38,
                0,
            ),
        ];
        for (strategy, max_attempts, attempts, successes, terminal_failures, denied, deferred) in
            golden
        {
            let report = run(
                &scenario,
                20261004,
                &AdmissionConfig {
                    strategy,
                    max_attempts,
                    base_delay: 1,
                    cap_delay: 32,
                    shared_budget: 1500,
                    refill_per_tick: 18,
                    failure_cost: 1,
                },
            );
            assert_eq!(
                report.summary.attempts, attempts,
                "attempts mismatch for {strategy:?}"
            );
            assert_eq!(
                report.summary.successes, successes,
                "successes mismatch for {strategy:?}"
            );
            assert_eq!(
                report.summary.terminal_failures, terminal_failures,
                "terminal_failures mismatch for {strategy:?}"
            );
            assert_eq!(
                report.summary.denied_retries, denied,
                "denied_retries mismatch for {strategy:?}"
            );
            assert_eq!(
                report.summary.deferred_retries, deferred,
                "deferred_retries mismatch for {strategy:?}"
            );
        }
    }
}
