//! Hexagonal Boundary: `RunFleetRetryWorkload`, the use case orchestrating a fleet
//! retry run generic over ports. Never names a concrete adapter crate — dependency
//! injection happens entirely through the `R`/`W`/`M` type parameters, supplied by a
//! composition root (`jitter-cli`, `jitter-service`).
//!
//! ADR-0110 (Application Layer Independence); docs/ddd/10-application-workflows.md.

use jitter_outcome::AdmissionStrategy;
use jitter_ports::{
    HarnessSink, MemorySink, PolicyProfile, ReceiptRef, RetryConfigSource, WitnessSink,
};
use jitter_scenario::ValidatedScenario;
use jitter_simulation::AdmissionConfig;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("retry config source: {0}")]
    RetryConfig(jitter_ports::PortError),
    #[error("witness sink: {0}")]
    Witness(jitter_ports::PortError),
    #[error("memory sink: {0}")]
    Memory(jitter_ports::PortError),
}

/// Resolves the `AdmissionConfig` a given `strategy` should run with, from any
/// `RetryConfigSource`. A free function (not a method) so a composition root can
/// call it without constructing a full `RunFleetRetryWorkload` first.
pub fn resolve_admission_config<R: RetryConfigSource>(
    retry_config: &R,
    strategy: AdmissionStrategy,
) -> Result<AdmissionConfig, AppError> {
    let profile = if strategy.budget_gated() {
        PolicyProfile::BudgetGated
    } else {
        PolicyProfile::Unbounded
    };
    let spec = retry_config
        .retry_budget(profile)
        .map_err(AppError::RetryConfig)?;
    Ok(AdmissionConfig {
        strategy,
        max_attempts: spec.max_attempts,
        base_delay: spec.base_delay,
        cap_delay: spec.cap_delay,
        shared_budget: spec.shared_budget,
        refill_per_tick: spec.refill_per_tick,
        failure_cost: spec.failure_cost,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunReceipt {
    pub receipt: jitter_evidence::Receipt,
    pub nearest_prior_ids: Vec<String>,
}

/// The use case: given a validated scenario, resolve a retry budget, run the
/// deterministic simulation kernel, seal the outcome with a witness, and persist a
/// receipt pointer to memory. Generic over every port; never names
/// `jitter-adapter-aws`, `jitter-adapter-rvf`, or `jitter-adapter-ruvector`.
pub struct RunFleetRetryWorkload<R: RetryConfigSource, W: WitnessSink, M: MemorySink> {
    retry_config: R,
    witness: W,
    memory: M,
    strategy: AdmissionStrategy,
    seed: u64,
}

impl<R: RetryConfigSource, W: WitnessSink, M: MemorySink> RunFleetRetryWorkload<R, W, M> {
    pub fn new(
        retry_config: R,
        witness: W,
        memory: M,
        strategy: AdmissionStrategy,
        seed: u64,
    ) -> Self {
        Self {
            retry_config,
            witness,
            memory,
            strategy,
            seed,
        }
    }

    /// The `AdmissionConfig` this workload would run with right now. Exposed so a
    /// composition root can drive `jitter-recovery::capture` directly for a
    /// simulated interruption, using the exact same resolved budget this workload's
    /// own `execute` would use.
    pub fn resolve_config(&self) -> Result<AdmissionConfig, AppError> {
        resolve_admission_config(&self.retry_config, self.strategy)
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Completes a [`SimulationRun`](jitter_simulation::SimulationRun) obtained
    /// out-of-band (e.g. from `jitter-recovery::replay`) through the same
    /// witness/evidence/memory pipeline `execute` uses.
    pub fn finish_run(
        &self,
        scenario: &ValidatedScenario,
        run: jitter_simulation::SimulationRun,
    ) -> Result<RunReceipt, AppError> {
        self.finish(scenario, run.tenant_original_requests, run.summary)
    }

    fn finish(
        &self,
        scenario: &ValidatedScenario,
        tenant_original_requests: Vec<u32>,
        summary: jitter_outcome::OutcomeSummary,
    ) -> Result<RunReceipt, AppError> {
        let witness_ref = self
            .witness
            .write(&summary, scenario.digest())
            .map_err(AppError::Witness)?;
        let receipt = jitter_evidence::bind(
            scenario.digest(),
            scenario.requests(),
            &tenant_original_requests,
            &summary,
            witness_ref.digest,
        );
        let receipt_ref = ReceiptRef {
            scenario_digest: receipt.scenario_digest.clone(),
            outcome_digest: receipt.outcome_digest.clone(),
            witness_digest: receipt.witness_digest.clone(),
            authority: receipt.authority.clone(),
            retry_amplification_factor: receipt.retry_amplification_factor,
            p95_ticks: receipt.p95_ticks,
            tenant_success_fairness: receipt.tenant_success_fairness,
        };
        let nearest_prior_ids = self.memory.record(&receipt_ref).map_err(AppError::Memory)?;
        Ok(RunReceipt {
            receipt,
            nearest_prior_ids,
        })
    }

    /// Runs `scenario` end to end with no orchestration-signaled interruption.
    pub fn execute(&self, scenario: ValidatedScenario) -> Result<RunReceipt, AppError> {
        let config = self.resolve_config()?;
        let run = jitter_simulation::run(&scenario, self.seed, &config);
        self.finish(&scenario, run.tenant_original_requests, run.summary)
    }

    /// Runs `scenario`, reporting every outcome event to `harness` as it is produced
    /// and, if `harness` reports an interruption signal before the run starts,
    /// routing through `jitter-recovery::capture` + `jitter-recovery::replay` instead
    /// of a plain `jitter-simulation::run` (docs/architecture.md §7). The replayed
    /// run must and does produce the same receipt a plain run would.
    pub fn execute_with_recovery<H: HarnessSink>(
        &self,
        scenario: ValidatedScenario,
        harness: &H,
    ) -> Result<RunReceipt, AppError> {
        let config = self.resolve_config()?;
        let (tenant_original_requests, events) = match harness.poll_interruption() {
            Some(signal) => {
                let checkpoint =
                    jitter_recovery::capture(&scenario, self.seed, &config, signal.at_tick);
                let remaining = jitter_workload::generate(&scenario, self.seed);
                let run = jitter_recovery::replay(&checkpoint, remaining);
                (run.tenant_original_requests, run.summary)
            }
            None => {
                let run = jitter_simulation::run(&scenario, self.seed, &config);
                (run.tenant_original_requests, run.summary)
            }
        };
        let _ = harness.on_tick(&synthetic_completion_event(&events));
        self.finish(&scenario, tenant_original_requests, events)
    }
}

fn synthetic_completion_event(
    summary: &jitter_outcome::OutcomeSummary,
) -> jitter_outcome::OutcomeEvent {
    let tick = summary.completed_at.iter().max().copied().unwrap_or(0);
    jitter_outcome::OutcomeEvent::Admitted {
        tenant_id: 0,
        request_id: 0,
        tick,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jitter_ports::RetryBudgetSpec;
    use jitter_ports::mocks::{
        MockHarnessSink, MockMemorySink, MockRetryConfigSource, MockWitnessSink,
    };
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

    fn workload(
        strategy: AdmissionStrategy,
    ) -> RunFleetRetryWorkload<MockRetryConfigSource, MockWitnessSink, MockMemorySink> {
        RunFleetRetryWorkload::new(
            MockRetryConfigSource {
                spec: RetryBudgetSpec {
                    max_attempts: 5,
                    base_delay: 1,
                    cap_delay: 16,
                    shared_budget: 40,
                    refill_per_tick: 2,
                    failure_cost: 3,
                    upstream_revision: "mock".into(),
                },
            },
            MockWitnessSink,
            MockMemorySink::default(),
            strategy,
            7,
        )
    }

    /// Row 10's acceptance criterion: a mocked E2E use-case test for
    /// `fleet-retry-storm-22`-shaped input passes with zero concrete adapters.
    #[test]
    fn mocked_e2e_use_case_passes_with_zero_concrete_adapters() {
        let workload = workload(AdmissionStrategy::AdaptiveBudget);
        let receipt = workload.execute(scenario()).unwrap();
        assert_eq!(receipt.receipt.authority, "none");
        assert!(receipt.receipt.retry_amplification_factor > 0.0);
        assert!(receipt.nearest_prior_ids.is_empty());
    }

    #[test]
    fn repeated_memory_record_returns_growing_nearest_prior_list() {
        let workload = workload(AdmissionStrategy::AdaptiveBudget);
        let first = workload.execute(scenario()).unwrap();
        let second = workload.execute(scenario()).unwrap();
        assert_eq!(first.nearest_prior_ids.len(), 0);
        assert_eq!(second.nearest_prior_ids.len(), 1);
    }

    #[test]
    fn port_failure_is_never_swallowed_into_success() {
        struct AlwaysFails;
        impl RetryConfigSource for AlwaysFails {
            fn retry_budget(
                &self,
                _: PolicyProfile,
            ) -> Result<RetryBudgetSpec, jitter_ports::PortError> {
                Err(jitter_ports::PortError::Unavailable(
                    "no budget source".into(),
                ))
            }
        }
        let workload = RunFleetRetryWorkload::new(
            AlwaysFails,
            MockWitnessSink,
            MockMemorySink::default(),
            AdmissionStrategy::AdaptiveBudget,
            7,
        );
        assert!(matches!(
            workload.execute(scenario()),
            Err(AppError::RetryConfig(_))
        ));
    }

    #[test]
    fn execute_without_interruption_matches_plain_execute() {
        let workload = workload(AdmissionStrategy::WeightedTenantFairBudget);
        let harness = MockHarnessSink { interrupt_at: None };
        let plain = workload.execute(scenario()).unwrap();
        let reported = workload
            .execute_with_recovery(scenario(), &harness)
            .unwrap();
        assert_eq!(
            plain.receipt.outcome_digest,
            reported.receipt.outcome_digest
        );
    }

    #[test]
    fn execute_with_recovery_routes_through_checkpoint_replay_on_interruption() {
        let workload = workload(AdmissionStrategy::AdaptiveBudget);
        let no_interrupt = MockHarnessSink { interrupt_at: None };
        let interrupted = MockHarnessSink {
            interrupt_at: Some(6),
        };
        let baseline = workload
            .execute_with_recovery(scenario(), &no_interrupt)
            .unwrap();
        let replayed = workload
            .execute_with_recovery(scenario(), &interrupted)
            .unwrap();
        assert_eq!(
            baseline.receipt.outcome_digest,
            replayed.receipt.outcome_digest
        );
    }
}
