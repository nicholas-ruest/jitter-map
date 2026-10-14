//! Domain Core: the consumable shared retry budget.
//!
//! ADR-0104 (Budget Value Object); docs/ddd/06-admission-budget.md.
//!
//! `BudgetExhausted` is the only failure mode; the budget's remaining balance never
//! goes negative (docs/implementation-contract.md).

use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Weight = u32;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("budget exhausted: needed {needed}, remaining {remaining}")]
pub struct BudgetExhausted {
    pub needed: Weight,
    pub remaining: Weight,
}

/// A shared, tick-refilled consumable budget. `cap` and `refill_per_tick` are fixed at
/// construction (sourced from `RetryConfigSource` via the composition root);
/// `remaining` and `last_tick` are the only mutable state, which is exactly the state a
/// [`Checkpoint`](../jitter_recovery/struct.Checkpoint.html) must snapshot to resume a
/// run bit-for-bit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budget {
    cap: Weight,
    refill_per_tick: Weight,
    remaining: Weight,
    last_tick: u32,
}

impl Budget {
    pub fn new(cap: Weight, refill_per_tick: Weight) -> Self {
        Self {
            cap,
            refill_per_tick,
            remaining: cap,
            last_tick: 0,
        }
    }

    pub fn remaining(&self) -> Weight {
        self.remaining
    }

    pub fn cap(&self) -> Weight {
        self.cap
    }

    /// Refills the budget for every tick elapsed since the last mutation, saturating at
    /// `cap`. Must be called before `try_consume` for a tick that has advanced.
    pub fn refill_through(&mut self, tick: u32) {
        if tick > self.last_tick {
            let elapsed = tick - self.last_tick;
            self.remaining = self
                .remaining
                .saturating_add(elapsed.saturating_mul(self.refill_per_tick))
                .min(self.cap);
            self.last_tick = tick;
        }
    }

    /// Consumes `cost` from the remaining balance, or fails closed with
    /// [`BudgetExhausted`] and no mutation.
    pub fn try_consume(&mut self, cost: Weight) -> Result<(), BudgetExhausted> {
        if cost > self.remaining {
            return Err(BudgetExhausted {
                needed: cost,
                remaining: self.remaining,
            });
        }
        self.remaining -= cost;
        Ok(())
    }

    /// Forces the remaining balance to zero. Used only by the adaptive-deferral
    /// admission strategy, which spends the entire remaining budget as the cost of
    /// deferring a request rather than charging just the request's own cost
    /// (ADR-0026).
    pub fn exhaust(&mut self) {
        self.remaining = 0;
    }

    /// The number of whole ticks of refill needed before `cost` could be covered,
    /// given the current shortfall. `None` if `refill_per_tick` is zero and the budget
    /// can never recover on its own.
    pub fn ticks_until_available(&self, cost: Weight) -> Option<u32> {
        if cost <= self.remaining {
            return Some(0);
        }
        if self.refill_per_tick == 0 {
            return None;
        }
        Some((cost - self.remaining).div_ceil(self.refill_per_tick))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_cap() {
        let b = Budget::new(10, 2);
        assert_eq!(b.remaining(), 10);
    }

    #[test]
    fn consume_reduces_balance() {
        let mut b = Budget::new(10, 2);
        b.try_consume(3).unwrap();
        assert_eq!(b.remaining(), 7);
    }

    #[test]
    fn exhaustion_leaves_balance_unchanged() {
        let mut b = Budget::new(5, 1);
        let err = b.try_consume(6).unwrap_err();
        assert_eq!(
            err,
            BudgetExhausted {
                needed: 6,
                remaining: 5
            }
        );
        assert_eq!(b.remaining(), 5);
    }

    #[test]
    fn balance_never_goes_negative() {
        let mut b = Budget::new(3, 0);
        assert!(b.try_consume(1).is_ok());
        assert!(b.try_consume(1).is_ok());
        assert!(b.try_consume(1).is_ok());
        assert!(b.try_consume(1).is_err());
        assert_eq!(b.remaining(), 0);
    }

    #[test]
    fn refill_is_capped() {
        let mut b = Budget::new(10, 5);
        b.try_consume(10).unwrap();
        b.refill_through(1);
        assert_eq!(b.remaining(), 5);
        b.refill_through(10);
        assert_eq!(b.remaining(), 10);
    }

    #[test]
    fn refill_is_idempotent_for_same_tick() {
        let mut b = Budget::new(10, 5);
        b.try_consume(10).unwrap();
        b.refill_through(1);
        b.refill_through(1);
        assert_eq!(b.remaining(), 5);
    }

    #[test]
    fn ticks_until_available_accounts_for_refill_rate() {
        let mut b = Budget::new(10, 2);
        b.try_consume(10).unwrap();
        assert_eq!(b.ticks_until_available(6), Some(3));
        assert_eq!(b.ticks_until_available(0), Some(0));
    }

    #[test]
    fn exhaust_forces_remaining_to_zero() {
        let mut b = Budget::new(10, 1);
        b.exhaust();
        assert_eq!(b.remaining(), 0);
    }

    #[test]
    fn ticks_until_available_is_none_without_refill() {
        let mut b = Budget::new(5, 0);
        b.try_consume(5).unwrap();
        assert_eq!(b.ticks_until_available(1), None);
    }
}
