//! Domain Core: typed outcome events, per-run aggregation, and the admission-strategy
//! identifier recorded on every run.
//!
//! ADR-0106 (Domain Event Log); docs/ddd/01-ubiquitous-language.md (Outcome Record).
//! `summarize` is infallible; its serde round-trip must be exact
//! (docs/implementation-contract.md).

use serde::{Deserialize, Serialize};

/// Which fleet-wide admission strategy produced a run. Carried on every
/// [`OutcomeSummary`] so a receipt is self-describing about which policy, budget-gating,
/// and ordering combination produced it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionStrategy {
    LocalJitter,
    AwsStandard,
    AdaptiveBudget,
    AdaptiveDeferral,
    TenantFairBudget,
    WeightedTenantFairBudget,
}

impl AdmissionStrategy {
    /// Whether retries under this strategy are gated by a consumable shared budget.
    pub fn budget_gated(self) -> bool {
        !matches!(
            self,
            AdmissionStrategy::LocalJitter | AdmissionStrategy::AwsStandard
        )
    }

    /// Whether a same-tick batch must be reordered into rotating per-tenant FIFO rounds
    /// before admission (equal-share or demand-weighted).
    pub fn fair_ordering(self) -> bool {
        matches!(
            self,
            AdmissionStrategy::TenantFairBudget | AdmissionStrategy::WeightedTenantFairBudget
        )
    }

    /// Whether demand weights (if present) should size each tenant's per-round quantum,
    /// versus a flat quantum of one.
    pub fn demand_weighted(self) -> bool {
        matches!(self, AdmissionStrategy::WeightedTenantFairBudget)
    }

    /// Whether budget exhaustion reschedules (defers) rather than terminally denying.
    pub fn deferral_enabled(self) -> bool {
        matches!(self, AdmissionStrategy::AdaptiveDeferral)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutcomeEvent {
    Admitted {
        tenant_id: u32,
        request_id: u32,
        tick: u32,
    },
    /// An attempt was made and will be retried later, but did not itself resolve as a
    /// success, deferral, denial, or exhaustion. Counts toward `attempts` only.
    Retried {
        tenant_id: u32,
        request_id: u32,
        tick: u32,
    },
    Deferred {
        tenant_id: u32,
        request_id: u32,
        tick: u32,
    },
    Denied {
        tenant_id: u32,
        request_id: u32,
        tick: u32,
    },
    Exhausted {
        tenant_id: u32,
        request_id: u32,
        tick: u32,
    },
    HorizonExceeded {
        tenant_id: u32,
        request_id: u32,
        tick: u32,
    },
}

impl OutcomeEvent {
    pub fn tenant_id(&self) -> u32 {
        match *self {
            OutcomeEvent::Admitted { tenant_id, .. }
            | OutcomeEvent::Retried { tenant_id, .. }
            | OutcomeEvent::Deferred { tenant_id, .. }
            | OutcomeEvent::Denied { tenant_id, .. }
            | OutcomeEvent::Exhausted { tenant_id, .. }
            | OutcomeEvent::HorizonExceeded { tenant_id, .. } => tenant_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeSummary {
    pub strategy: AdmissionStrategy,
    pub attempts: u64,
    pub successes: u32,
    pub terminal_failures: u32,
    pub denied_retries: u64,
    pub deferred_retries: u64,
    pub tenant_attempts: Vec<u64>,
    pub tenant_successes: Vec<u32>,
    pub tenant_terminal_failures: Vec<u32>,
    pub tenant_denied_retries: Vec<u64>,
    pub tenant_deferred_retries: Vec<u64>,
    pub completed_at: Vec<u32>,
}

/// Aggregates a flat event log into one [`OutcomeSummary`]. `tenants` is the fleet size
/// so that tenants with zero events still have a zeroed slot in every per-tenant vector.
///
/// Horizon-exceeded events count toward `terminal_failures` but not `attempts`: a
/// request whose retry fell beyond the simulation horizon was never actually attempted
/// against the backend (it is dropped before an attempt is spent), matching the base
/// `3db7fd2` semantics this crate reproduces bit-for-bit.
pub fn summarize(events: &[OutcomeEvent], tenants: u32) -> OutcomeSummary {
    let n = tenants as usize;
    let mut summary = OutcomeSummary {
        strategy: AdmissionStrategy::AdaptiveBudget,
        attempts: 0,
        successes: 0,
        terminal_failures: 0,
        denied_retries: 0,
        deferred_retries: 0,
        tenant_attempts: vec![0; n],
        tenant_successes: vec![0; n],
        tenant_terminal_failures: vec![0; n],
        tenant_denied_retries: vec![0; n],
        tenant_deferred_retries: vec![0; n],
        completed_at: Vec::new(),
    };
    for event in events {
        let tenant = event.tenant_id() as usize;
        match *event {
            OutcomeEvent::Admitted { tick, .. } => {
                summary.attempts += 1;
                summary.tenant_attempts[tenant] += 1;
                summary.successes += 1;
                summary.tenant_successes[tenant] += 1;
                summary.completed_at.push(tick);
            }
            OutcomeEvent::Retried { .. } => {
                summary.attempts += 1;
                summary.tenant_attempts[tenant] += 1;
            }
            OutcomeEvent::Deferred { .. } => {
                summary.attempts += 1;
                summary.tenant_attempts[tenant] += 1;
                summary.deferred_retries += 1;
                summary.tenant_deferred_retries[tenant] += 1;
            }
            OutcomeEvent::Denied { .. } => {
                summary.attempts += 1;
                summary.tenant_attempts[tenant] += 1;
                summary.denied_retries += 1;
                summary.tenant_denied_retries[tenant] += 1;
                summary.terminal_failures += 1;
                summary.tenant_terminal_failures[tenant] += 1;
            }
            OutcomeEvent::Exhausted { .. } => {
                summary.attempts += 1;
                summary.tenant_attempts[tenant] += 1;
                summary.terminal_failures += 1;
                summary.tenant_terminal_failures[tenant] += 1;
            }
            OutcomeEvent::HorizonExceeded { .. } => {
                summary.terminal_failures += 1;
                summary.tenant_terminal_failures[tenant] += 1;
            }
        }
    }
    summary
}

/// Sets the strategy tag on a summary built by [`summarize`] (kept as a separate step so
/// `summarize` itself stays strategy-agnostic and testable in isolation).
pub fn with_strategy(mut summary: OutcomeSummary, strategy: AdmissionStrategy) -> OutcomeSummary {
    summary.strategy = strategy;
    summary
}

pub fn retry_amplification_factor(attempts: u64, original_requests: u32) -> f64 {
    attempts as f64 / f64::from(original_requests)
}

pub fn p95_ticks(completed_at: &[u32]) -> u32 {
    quantile(completed_at, 95)
}

/// Jain fairness index over per-tenant success ratios (successes / original requests).
/// `None` when no tenant has a defined ratio — undefined fairness stays explicit rather
/// than defaulting to a misleading number.
pub fn tenant_success_fairness(
    tenant_original_requests: &[u32],
    tenant_successes: &[u32],
) -> Option<f64> {
    let ratios: Vec<f64> = tenant_original_requests
        .iter()
        .zip(tenant_successes)
        .filter_map(|(&requests, &successes)| {
            (requests > 0).then_some(f64::from(successes) / f64::from(requests))
        })
        .collect();
    let sum: f64 = ratios.iter().sum();
    let sum_squares: f64 = ratios.iter().map(|value| value * value).sum();
    if ratios.is_empty() || sum_squares == 0.0 {
        None
    } else {
        Some(sum * sum / (ratios.len() as f64 * sum_squares))
    }
}

fn quantile(values: &[u32], pct: usize) -> u32 {
    if values.is_empty() {
        return 0;
    }
    let mut v = values.to_vec();
    v.sort_unstable();
    v[((v.len() - 1) * pct / 100).min(v.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admitted_counts_as_attempt_and_success() {
        let events = [OutcomeEvent::Admitted {
            tenant_id: 0,
            request_id: 1,
            tick: 5,
        }];
        let s = summarize(&events, 1);
        assert_eq!(s.attempts, 1);
        assert_eq!(s.successes, 1);
        assert_eq!(s.completed_at, vec![5]);
    }

    #[test]
    fn retried_counts_as_attempt_only() {
        let events = [OutcomeEvent::Retried {
            tenant_id: 0,
            request_id: 1,
            tick: 5,
        }];
        let s = summarize(&events, 1);
        assert_eq!(s.attempts, 1);
        assert_eq!(s.successes, 0);
        assert_eq!(s.terminal_failures, 0);
        assert_eq!(s.deferred_retries, 0);
        assert_eq!(s.denied_retries, 0);
    }

    #[test]
    fn horizon_exceeded_does_not_count_as_attempt() {
        let events = [OutcomeEvent::HorizonExceeded {
            tenant_id: 0,
            request_id: 1,
            tick: 500,
        }];
        let s = summarize(&events, 1);
        assert_eq!(s.attempts, 0);
        assert_eq!(s.terminal_failures, 1);
    }

    #[test]
    fn denied_is_both_denied_and_terminal() {
        let events = [OutcomeEvent::Denied {
            tenant_id: 0,
            request_id: 1,
            tick: 1,
        }];
        let s = summarize(&events, 1);
        assert_eq!(s.denied_retries, 1);
        assert_eq!(s.terminal_failures, 1);
        assert_eq!(s.attempts, 1);
    }

    #[test]
    fn deferred_is_not_terminal() {
        let events = [OutcomeEvent::Deferred {
            tenant_id: 0,
            request_id: 1,
            tick: 1,
        }];
        let s = summarize(&events, 1);
        assert_eq!(s.deferred_retries, 1);
        assert_eq!(s.terminal_failures, 0);
    }

    #[test]
    fn zero_event_tenants_still_get_a_slot() {
        let events = [OutcomeEvent::Admitted {
            tenant_id: 0,
            request_id: 1,
            tick: 1,
        }];
        let s = summarize(&events, 3);
        assert_eq!(s.tenant_attempts.len(), 3);
        assert_eq!(s.tenant_attempts, vec![1, 0, 0]);
    }

    #[test]
    fn serde_round_trip_is_exact() {
        let events = [
            OutcomeEvent::Admitted {
                tenant_id: 0,
                request_id: 1,
                tick: 1,
            },
            OutcomeEvent::Denied {
                tenant_id: 1,
                request_id: 2,
                tick: 2,
            },
        ];
        let s = with_strategy(summarize(&events, 2), AdmissionStrategy::TenantFairBudget);
        let json = serde_json::to_string(&s).unwrap();
        let back: OutcomeSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(s, back);
    }

    #[test]
    fn fairness_is_none_when_no_tenant_has_requests() {
        assert_eq!(tenant_success_fairness(&[0, 0], &[0, 0]), None);
    }

    #[test]
    fn fairness_is_one_when_equal_ratios() {
        let fairness = tenant_success_fairness(&[10, 10], &[5, 5]).unwrap();
        assert!((fairness - 1.0).abs() < 1e-9);
    }

    #[test]
    fn p95_of_empty_is_zero() {
        assert_eq!(p95_ticks(&[]), 0);
    }

    #[test]
    fn strategy_flags_match_admission_semantics() {
        assert!(!AdmissionStrategy::LocalJitter.budget_gated());
        assert!(AdmissionStrategy::AdaptiveBudget.budget_gated());
        assert!(AdmissionStrategy::WeightedTenantFairBudget.fair_ordering());
        assert!(AdmissionStrategy::WeightedTenantFairBudget.demand_weighted());
        assert!(!AdmissionStrategy::TenantFairBudget.demand_weighted());
        assert!(AdmissionStrategy::AdaptiveDeferral.deferral_enabled());
    }
}
