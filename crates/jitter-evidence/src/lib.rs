//! Cross-Cutting Support: the typed `Receipt` binding a scenario digest, a
//! summarized outcome, and a witness digest into one verifiable record.
//!
//! ADR-0116 (Evidence Receipt Model); docs/ddd/07-evidence-receipts.md. `bind` is a
//! pure function; a `Receipt` once bound is never mutated, only superseded by a new
//! one (docs/implementation-contract.md).
//!
//! **Documented deviation**: docs/architecture.md §5 shows `bind` consuming the
//! `WitnessRef` type from `jitter-adapter-rvf`/`jitter-ports`. Naming `jitter-ports`
//! here would cross a dependency edge docs/architecture.md §3 never draws (only
//! `EVID --> OUT` is shown) — this crate depends on Domain Core (`jitter-outcome`)
//! only. `bind` instead takes the witness digest as a plain `String`;
//! `jitter-application`, which already depends on both `jitter-ports` and
//! `jitter-evidence`, narrows `WitnessRef` to that string before calling `bind`.

use jitter_outcome::{AdmissionStrategy, OutcomeSummary};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    pub scenario_digest: String,
    pub outcome_digest: String,
    pub witness_digest: String,
    pub strategy: AdmissionStrategy,
    pub retry_amplification_factor: f64,
    pub p95_ticks: u32,
    pub tenant_success_fairness: Option<f64>,
    pub authority: String,
}

/// Binds a `Receipt`. The `outcome_digest` is a SHA-256 of the summary's canonical
/// JSON serialization, so the receipt stays independently re-verifiable without
/// re-running the simulation (row 16's acceptance criterion): anyone holding the
/// `OutcomeSummary` can recompute `outcome_digest` and compare.
pub fn bind(
    scenario_digest: &str,
    original_requests: u32,
    tenant_original_requests: &[u32],
    summary: &OutcomeSummary,
    witness_digest: String,
) -> Receipt {
    let outcome_digest = outcome_summary_digest(summary);
    Receipt {
        scenario_digest: scenario_digest.to_string(),
        outcome_digest,
        witness_digest,
        strategy: summary.strategy,
        retry_amplification_factor: jitter_outcome::retry_amplification_factor(
            summary.attempts,
            original_requests,
        ),
        p95_ticks: jitter_outcome::p95_ticks(&summary.completed_at),
        tenant_success_fairness: jitter_outcome::tenant_success_fairness(
            tenant_original_requests,
            &summary.tenant_successes,
        ),
        authority: "none".to_string(),
    }
}

pub fn outcome_summary_digest(summary: &OutcomeSummary) -> String {
    let json = serde_json::to_string(summary).expect("OutcomeSummary always serializes");
    let mut hasher = Sha256::new();
    hasher.update(json.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jitter_outcome::OutcomeEvent;

    fn summary() -> OutcomeSummary {
        jitter_outcome::with_strategy(
            jitter_outcome::summarize(
                &[
                    OutcomeEvent::Admitted {
                        tenant_id: 0,
                        request_id: 1,
                        tick: 2,
                    },
                    OutcomeEvent::Denied {
                        tenant_id: 1,
                        request_id: 2,
                        tick: 2,
                    },
                ],
                2,
            ),
            AdmissionStrategy::AdaptiveBudget,
        )
    }

    #[test]
    fn authority_is_always_none() {
        let receipt = bind("digest", 2, &[1, 1], &summary(), "witness".into());
        assert_eq!(receipt.authority, "none");
    }

    #[test]
    fn outcome_digest_is_stable_and_reverifiable() {
        let receipt = bind("digest", 2, &[1, 1], &summary(), "witness".into());
        assert_eq!(receipt.outcome_digest, outcome_summary_digest(&summary()));
    }

    #[test]
    fn outcome_digest_changes_with_summary() {
        let a = bind("digest", 2, &[1, 1], &summary(), "witness".into());
        let mut other = summary();
        other.attempts += 1;
        let b = bind("digest", 2, &[1, 1], &other, "witness".into());
        assert_ne!(a.outcome_digest, b.outcome_digest);
    }

    #[test]
    fn receipt_is_not_mutated_in_place_only_superseded() {
        let first = bind("digest", 2, &[1, 1], &summary(), "witness-1".into());
        let second = bind("digest", 2, &[1, 1], &summary(), "witness-2".into());
        assert_ne!(first.witness_digest, second.witness_digest);
        assert_eq!(first.outcome_digest, second.outcome_digest);
    }

    #[test]
    fn serde_round_trip_is_exact() {
        let receipt = bind("digest", 2, &[1, 1], &summary(), "witness".into());
        let json = serde_json::to_string(&receipt).unwrap();
        let back: Receipt = serde_json::from_str(&json).unwrap();
        assert_eq!(receipt, back);
    }
}
