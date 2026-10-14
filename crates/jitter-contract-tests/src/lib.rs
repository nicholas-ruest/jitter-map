//! Contract Verification: one parametrized assertion function per port trait in
//! `jitter-ports`. Every `jitter-adapter-*` crate invokes its corresponding
//! `assert_*` function from its own `#[cfg(test)]` suite; `jitter-ports`' mocks invoke
//! all five from theirs (ADR-0122; docs/ddd/02-context-map.md).
//!
//! This crate is dev-only everywhere it is used: it must never appear outside
//! `[dev-dependencies]` in any workspace manifest (docs/architecture.md §8 Gate 1).

use jitter_outcome::{AdmissionStrategy, OutcomeEvent, OutcomeSummary};
use jitter_ports::{
    CandidateResult, FlywheelSink, HarnessSink, MemorySink, PolicyProfile, ReceiptRef,
    RetryConfigSource, WitnessSink,
};

pub fn assert_retry_config_source<T: RetryConfigSource>(adapter: &T) {
    let gated = adapter
        .retry_budget(PolicyProfile::BudgetGated)
        .expect("a RetryConfigSource must resolve a budget-gated profile");
    assert!(gated.max_attempts > 0, "max_attempts must be positive");
    assert!(
        gated.base_delay > 0 && gated.base_delay <= gated.cap_delay,
        "base_delay must be positive and at most cap_delay"
    );
    assert!(gated.failure_cost > 0, "failure_cost must be positive");
    assert!(
        !gated.upstream_revision.is_empty(),
        "upstream_revision must carry real provenance, not an empty string"
    );

    let unbounded = adapter
        .retry_budget(PolicyProfile::Unbounded)
        .expect("a RetryConfigSource must also resolve an unbounded profile");
    assert!(unbounded.max_attempts > 0, "max_attempts must be positive");
}

fn sample_summary() -> OutcomeSummary {
    jitter_outcome::with_strategy(
        jitter_outcome::summarize(
            &[
                OutcomeEvent::Admitted {
                    tenant_id: 0,
                    request_id: 1,
                    tick: 1,
                },
                OutcomeEvent::Denied {
                    tenant_id: 1,
                    request_id: 2,
                    tick: 1,
                },
            ],
            2,
        ),
        AdmissionStrategy::AdaptiveBudget,
    )
}

pub fn assert_witness_sink<T: WitnessSink>(adapter: &T) {
    let summary = sample_summary();
    let first = adapter
        .write(&summary, "digest-a")
        .expect("a WitnessSink must seal a well-formed outcome");
    let second = adapter
        .write(&summary, "digest-a")
        .expect("a WitnessSink must seal the same input again");
    assert_eq!(
        first, second,
        "witness digest must be byte-stable for identical input"
    );
    assert!(!first.digest.is_empty(), "witness digest must not be empty");

    let different = adapter
        .write(&summary, "digest-b")
        .expect("a WitnessSink must seal a different scenario digest");
    assert_ne!(
        first, different,
        "witness digest must change when scenario digest changes"
    );
}

fn sample_receipt() -> ReceiptRef {
    ReceiptRef {
        scenario_digest: "a".repeat(64),
        outcome_digest: "b".repeat(64),
        witness_digest: "c".repeat(64),
        authority: "none".into(),
        retry_amplification_factor: 1.5,
        p95_ticks: 10,
        tenant_success_fairness: Some(0.9),
    }
}

pub fn assert_memory_sink<T: MemorySink>(adapter: &T) {
    let receipt = sample_receipt();
    let first = adapter
        .record(&receipt)
        .expect("a MemorySink must accept a well-formed receipt");
    let second = adapter
        .record(&receipt)
        .expect("a MemorySink must accept a repeated receipt");
    assert!(
        second.len() >= first.len(),
        "nearest-prior recall must not shrink after another record"
    );
}

pub fn assert_harness_sink<T: HarnessSink>(adapter: &T) {
    let event = OutcomeEvent::Admitted {
        tenant_id: 0,
        request_id: 0,
        tick: 0,
    };
    adapter
        .on_tick(&event)
        .expect("a HarnessSink must accept a well-formed tick report");
    // Either Some or None is a legitimate answer; the contract is "does not panic".
    let _ = adapter.poll_interruption();
}

pub fn assert_flywheel_sink<T: FlywheelSink>(adapter: &T) {
    let result = CandidateResult {
        candidate_id: "candidate-a".into(),
        fitness: 0.5,
        scenario_digest: "a".repeat(64),
    };
    adapter
        .submit(&result)
        .expect("a FlywheelSink must accept a well-formed candidate result");
}
