//! Domain Core: pure retry-delay decision rules.
//!
//! ADR-0103 (Policy Strategy Pattern); docs/ddd/04-policy-model.md.
//!
//! A `RetryPolicy` is a pure function: no I/O, no clock, no error variant. A policy
//! that cannot decide is a bug, not a `Result` (docs/implementation-contract.md).

use jitter_scenario::FailureClass;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AttemptHistory {
    pub request_id: u32,
    pub tenant_id: u32,
    pub tick: u32,
    pub attempt: u32,
    pub seed: u64,
    pub failure_class: FailureClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Retry the request at the given future tick.
    RetryAt(u32),
    /// No further attempt is permitted: either the attempt ceiling was reached, or the
    /// scenario's failure class is permanent regardless of attempts remaining.
    Exhausted,
}

pub trait RetryPolicy {
    fn next_delay(&self, history: &AttemptHistory) -> Decision;
}

/// Seeded full-jitter exponential backoff (ADR-0007). Shared by every admission
/// strategy's delay computation; what differs between strategies is budget-gating and
/// scheduling order, both owned by `jitter-budget` and `jitter-scheduler`.
#[derive(Debug, Clone, Copy)]
pub struct ExponentialJitter {
    pub base_delay: u32,
    pub cap_delay: u32,
    pub max_attempts: u32,
}

impl RetryPolicy for ExponentialJitter {
    fn next_delay(&self, history: &AttemptHistory) -> Decision {
        if history.attempt >= self.max_attempts || history.failure_class == FailureClass::Permanent
        {
            return Decision::Exhausted;
        }
        let ceiling = self
            .base_delay
            .saturating_mul(
                1u32.checked_shl(history.attempt.saturating_sub(1))
                    .unwrap_or(u32::MAX),
            )
            .min(self.cap_delay);
        let delay = 1
            + (random_word(history.seed, history.request_id, history.attempt) % u64::from(ceiling))
                as u32;
        Decision::RetryAt(history.tick.saturating_add(delay))
    }
}

/// Delay strategy used by the tenant-fair admission family (`TenantFairBudget`,
/// `WeightedTenantFairBudget`). The delay math is identical to [`ExponentialJitter`] —
/// the fairness property these strategies add comes from `jitter-scheduler`'s ordering
/// and `jitter-budget`'s consumption, not from a different delay curve. `DemandWeighted`
/// exists as a distinct type so a caller's choice of policy and choice of ordering
/// strategy stay independently named and cannot be silently conflated.
#[derive(Debug, Clone, Copy)]
pub struct DemandWeighted(pub ExponentialJitter);

impl RetryPolicy for DemandWeighted {
    fn next_delay(&self, history: &AttemptHistory) -> Decision {
        self.0.next_delay(history)
    }
}

fn random_word(seed: u64, id: u32, attempt: u32) -> u64 {
    let mut x = seed ^ u64::from(id).wrapping_mul(0x9e3779b97f4a7c15) ^ u64::from(attempt);
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history(attempt: u32, failure: FailureClass) -> AttemptHistory {
        AttemptHistory {
            request_id: 3,
            tenant_id: 1,
            tick: 10,
            attempt,
            seed: 99,
            failure_class: failure,
        }
    }

    #[test]
    fn exhausted_at_max_attempts() {
        let p = ExponentialJitter {
            base_delay: 1,
            cap_delay: 16,
            max_attempts: 5,
        };
        assert_eq!(
            p.next_delay(&history(5, FailureClass::Throttling)),
            Decision::Exhausted
        );
    }

    #[test]
    fn permanent_failure_is_always_exhausted() {
        let p = ExponentialJitter {
            base_delay: 1,
            cap_delay: 16,
            max_attempts: 5,
        };
        assert_eq!(
            p.next_delay(&history(1, FailureClass::Permanent)),
            Decision::Exhausted
        );
    }

    #[test]
    fn retry_delay_is_bounded_by_cap() {
        let p = ExponentialJitter {
            base_delay: 1,
            cap_delay: 4,
            max_attempts: 10,
        };
        for attempt in 1..10 {
            let Decision::RetryAt(next) = p.next_delay(&history(attempt, FailureClass::Timeout))
            else {
                panic!("expected retry")
            };
            assert!(next <= 10 + 4);
            assert!(next > 10);
        }
    }

    #[test]
    fn decision_is_deterministic_for_same_inputs() {
        let p = ExponentialJitter {
            base_delay: 2,
            cap_delay: 32,
            max_attempts: 6,
        };
        let h = history(2, FailureClass::Throttling);
        assert_eq!(p.next_delay(&h), p.next_delay(&h));
    }

    #[test]
    fn demand_weighted_matches_underlying_jitter() {
        let inner = ExponentialJitter {
            base_delay: 1,
            cap_delay: 16,
            max_attempts: 5,
        };
        let weighted = DemandWeighted(inner);
        let h = history(2, FailureClass::Throttling);
        assert_eq!(weighted.next_delay(&h), inner.next_delay(&h));
    }

    #[test]
    fn different_attempts_vary_delay_within_bound() {
        let p = ExponentialJitter {
            base_delay: 1,
            cap_delay: 32,
            max_attempts: 10,
        };
        let d1 = p.next_delay(&history(1, FailureClass::Throttling));
        let d2 = p.next_delay(&history(3, FailureClass::Throttling));
        assert_ne!(d1, d2);
    }
}
