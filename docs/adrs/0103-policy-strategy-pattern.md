# ADR-0103: Policy strategy pattern (`jitter-policy`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13
- Extends: ADR-0021

## Context

The base monolith's `RetryPolicy` struct conflated three orthogonal concerns behind
one `PolicyKind` match: the delay curve (exponential jitter), whether a shared budget
gates admission, and whether a same-tick batch is reordered for fairness. ADR-0027
planned `jitter-policy` to own only the delay decision, as a pure strategy.

## Viable alternatives

- Keep one `PolicyKind` enum driving all three concerns in one `match`. Rejected: this
  is exactly the conflation ADR-0027 planned to split apart; it makes `jitter-budget`
  and `jitter-scheduler` unable to exist as independently testable crates.
- Split delay, budget-gating, and ordering into three crates but give `DemandWeighted`
  a different delay curve than `ExponentialJitter`. Rejected: the base monolith's
  demand-weighted fairness comes entirely from scheduler ordering and budget
  consumption, not from a different backoff shape — inventing a different curve would
  break row 3's bit-for-bit reproduction requirement.
- Split the three concerns across `jitter-policy`/`jitter-budget`/`jitter-scheduler`,
  with `DemandWeighted` wrapping the same `ExponentialJitter` math under a distinct
  type. Chosen.

## Decision

`RetryPolicy::next_delay(&AttemptHistory) -> Decision` is pure: no I/O, no error
variant. `ExponentialJitter` implements the seeded full-jitter curve (ADR-0007).
`DemandWeighted` wraps an `ExponentialJitter` and delegates — it exists so a caller's
choice of delay strategy and choice of ordering/budget strategy
(`AdmissionStrategy` in `jitter-outcome`) stay independently named types, never
silently conflated back into one enum.

## Tradeoffs

Having `DemandWeighted` produce identical delays to `ExponentialJitter` can look like
a pointless wrapper. It is not: it is the type-level marker that keeps "which delay
curve" and "which fairness strategy" from re-merging into the single conflated
`PolicyKind` this ADR exists to retire.

## Consequences

`jitter-policy` depends only on `jitter-scenario` (for `FailureClass`, used to detect
permanent-failure exhaustion regardless of attempt count).

## Validation

- `cargo test -p jitter-policy` (6 tests: exhaustion at max attempts, permanent-failure
  exhaustion, delay bounded by cap, determinism, `DemandWeighted` matching its inner
  `ExponentialJitter` exactly)
- `jitter-simulation`'s golden-value regression test reproduces the base monolith's
  admission output bit-for-bit across all six strategies using this crate's two policy
  types
