//! Hexagonal Boundary: the five outbound port traits. Trait-only; zero
//! implementations in this crate outside its own `#[cfg(test)]` mocks.
//!
//! ADR-0109 (Ports & Adapters Boundary); docs/ddd/02-context-map.md,
//! docs/ddd/09-upstream-integrations.md.
//!
//! **Documented deviation from docs/implementation-contract.md's planning-table
//! signatures**: `WitnessSink::write` and `MemorySink::record` are shown there taking
//! `&SimulationRun` / `&RunReceipt`. Both of those types live in Domain Orchestration
//! (`jitter-simulation`) and Cross-Cutting Support (`jitter-evidence`) respectively —
//! naming either here would give this crate an outward dependency edge that
//! docs/architecture.md §3 makes structurally forbidden (Gate 1). Each port instead
//! takes the smaller DTO it actually needs ([`OutcomeSummary`] plus a digest string,
//! or [`ReceiptRef`], a port-local value type). `jitter-application`, which already
//! depends on every one of those crates, is responsible for narrowing before calling
//! through the port.

use jitter_outcome::{OutcomeEvent, OutcomeSummary};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PortError {
    #[error("upstream unavailable: {0}")]
    Unavailable(String),
    #[error("upstream rejected request: {0}")]
    Rejected(String),
}

/// Which attempt-budget shape a caller needs: an admission strategy with no shared
/// budget (`Unbounded`) or one gated by a consumable shared budget (`BudgetGated`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyProfile {
    Unbounded,
    BudgetGated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RetryBudgetSpec {
    pub max_attempts: u32,
    pub base_delay: u32,
    pub cap_delay: u32,
    pub shared_budget: u32,
    pub refill_per_tick: u32,
    pub failure_cost: u32,
    pub upstream_revision: String,
}

/// Resolves the retry-budget parameters a composition root should run with, sourced
/// from an external retry-configuration authority (e.g. AWS `RetryConfig`,
/// ADR-0005/ADR-0111).
pub trait RetryConfigSource {
    fn retry_budget(&self, profile: PolicyProfile) -> Result<RetryBudgetSpec, PortError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WitnessRef {
    pub digest: String,
}

/// Seals a run's outcome into a tamper-evident witness (RVF, ADR-0009/ADR-0113).
pub trait WitnessSink {
    fn write(
        &self,
        outcome: &OutcomeSummary,
        scenario_digest: &str,
    ) -> Result<WitnessRef, PortError>;
}

/// A port-local stand-in for `jitter-evidence::Receipt`, narrow enough that
/// `jitter-ports` never has to name the evidence crate. `jitter-application` builds
/// one of these from a real `Receipt` before calling through [`MemorySink`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiptRef {
    pub scenario_digest: String,
    pub outcome_digest: String,
    pub witness_digest: String,
    pub authority: String,
    pub retry_amplification_factor: f64,
    pub p95_ticks: u32,
    pub tenant_success_fairness: Option<f64>,
}

/// Persists a receipt pointer and recalls nearest-prior outcomes (RuVector,
/// ADR-0008/ADR-0112).
pub trait MemorySink {
    fn record(&self, receipt: &ReceiptRef) -> Result<Vec<String>, PortError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InterruptionSignal {
    pub at_tick: u32,
}

/// Reports tick progress to, and receives interruption signals from, an external
/// orchestration/coordination layer (Ruflo, ADR-0114). Driving `jitter-recovery` from
/// a real interruption is this port's reason to exist — there is no equivalent in the
/// base six-package implementation.
pub trait HarnessSink {
    fn on_tick(&self, event: &OutcomeEvent) -> Result<(), PortError>;
    fn poll_interruption(&self) -> Option<InterruptionSignal>;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CandidateResult {
    pub candidate_id: String,
    pub fitness: f64,
    pub scenario_digest: String,
}

/// Bridges `jitter-evaluation`'s scoring to the Darwin/Flywheel feedback loop
/// (ADR-0023/ADR-0115). Advisory only — nothing on the other side of this port may
/// promote a candidate (ADR-0010, ADR-0022).
pub trait FlywheelSink {
    fn submit(&self, result: &CandidateResult) -> Result<(), PortError>;
}

/// Minimal in-memory implementations of every port trait. Not test-gated: a
/// downstream crate's own test suite (`jitter-application`'s mocked E2E use-case
/// test, row 10's acceptance criterion) needs to depend on these from its own
/// `#[cfg(test)]` code, which only works if `jitter-ports` ships them as a normal
/// public module rather than one gated by its own `cfg(test)` (cfg(test) items never
/// cross a crate boundary, mocked or not).
pub mod mocks {
    use super::*;
    use std::sync::Mutex;

    pub struct MockRetryConfigSource {
        pub spec: RetryBudgetSpec,
    }
    impl RetryConfigSource for MockRetryConfigSource {
        fn retry_budget(&self, _profile: PolicyProfile) -> Result<RetryBudgetSpec, PortError> {
            Ok(self.spec.clone())
        }
    }

    #[derive(Default)]
    pub struct MockWitnessSink;
    impl WitnessSink for MockWitnessSink {
        fn write(
            &self,
            outcome: &OutcomeSummary,
            scenario_digest: &str,
        ) -> Result<WitnessRef, PortError> {
            Ok(WitnessRef {
                digest: format!("{scenario_digest}:{}", outcome.attempts),
            })
        }
    }

    #[derive(Default)]
    pub struct MockMemorySink {
        pub stored: Mutex<Vec<ReceiptRef>>,
    }
    impl MemorySink for MockMemorySink {
        fn record(&self, receipt: &ReceiptRef) -> Result<Vec<String>, PortError> {
            let mut stored = self.stored.lock().expect("mock memory lock");
            let prior: Vec<String> = stored.iter().map(|r| r.scenario_digest.clone()).collect();
            stored.push(receipt.clone());
            Ok(prior)
        }
    }

    #[derive(Default)]
    pub struct MockHarnessSink {
        pub interrupt_at: Option<u32>,
    }
    impl HarnessSink for MockHarnessSink {
        fn on_tick(&self, _event: &OutcomeEvent) -> Result<(), PortError> {
            Ok(())
        }
        fn poll_interruption(&self) -> Option<InterruptionSignal> {
            self.interrupt_at
                .map(|at_tick| InterruptionSignal { at_tick })
        }
    }

    #[derive(Default)]
    pub struct MockFlywheelSink {
        pub submitted: Mutex<Vec<CandidateResult>>,
    }
    impl FlywheelSink for MockFlywheelSink {
        fn submit(&self, result: &CandidateResult) -> Result<(), PortError> {
            self.submitted
                .lock()
                .expect("mock flywheel lock")
                .push(result.clone());
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::mocks::*;
    use super::*;

    fn sample_summary() -> OutcomeSummary {
        jitter_outcome::summarize(
            &[OutcomeEvent::Admitted {
                tenant_id: 0,
                request_id: 1,
                tick: 1,
            }],
            1,
        )
    }

    #[test]
    fn mock_retry_config_source_resolves_both_profiles() {
        let adapter = MockRetryConfigSource {
            spec: RetryBudgetSpec {
                max_attempts: 6,
                base_delay: 1,
                cap_delay: 32,
                shared_budget: 1500,
                refill_per_tick: 18,
                failure_cost: 1,
                upstream_revision: "mock".into(),
            },
        };
        assert!(adapter.retry_budget(PolicyProfile::BudgetGated).is_ok());
        assert!(adapter.retry_budget(PolicyProfile::Unbounded).is_ok());
    }

    #[test]
    fn mock_witness_sink_is_stable_and_digest_binds_scenario() {
        let summary = sample_summary();
        let a = MockWitnessSink.write(&summary, "digest-a").unwrap();
        let b = MockWitnessSink.write(&summary, "digest-a").unwrap();
        assert_eq!(a, b);
        let c = MockWitnessSink.write(&summary, "digest-b").unwrap();
        assert_ne!(a, c);
    }

    #[test]
    fn mock_memory_sink_recalls_prior_receipts() {
        let sink = MockMemorySink::default();
        let receipt = ReceiptRef {
            scenario_digest: "a".repeat(64),
            outcome_digest: "b".repeat(64),
            witness_digest: "c".repeat(64),
            authority: "none".into(),
            retry_amplification_factor: 1.2,
            p95_ticks: 5,
            tenant_success_fairness: Some(0.8),
        };
        assert_eq!(sink.record(&receipt).unwrap().len(), 0);
        assert_eq!(sink.record(&receipt).unwrap().len(), 1);
    }

    #[test]
    fn mock_harness_sink_reports_interruption() {
        let sink = MockHarnessSink {
            interrupt_at: Some(6),
        };
        let event = OutcomeEvent::Admitted {
            tenant_id: 0,
            request_id: 0,
            tick: 0,
        };
        assert!(sink.on_tick(&event).is_ok());
        assert_eq!(
            sink.poll_interruption(),
            Some(InterruptionSignal { at_tick: 6 })
        );
    }

    #[test]
    fn mock_flywheel_sink_accepts_candidate_result() {
        let sink = MockFlywheelSink::default();
        let result = CandidateResult {
            candidate_id: "candidate-a".into(),
            fitness: 0.9,
            scenario_digest: "a".repeat(64),
        };
        assert!(sink.submit(&result).is_ok());
        assert_eq!(sink.submitted.lock().unwrap().len(), 1);
    }
}
