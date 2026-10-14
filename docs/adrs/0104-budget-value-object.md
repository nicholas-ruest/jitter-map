# ADR-0104: Budget value object (`jitter-budget`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-budget` to own quantum/rotation bookkeeping, separate from
policy decision. In the base monolith, `budget` was a single `u32` mutated inline
inside `simulate`, with refill, consumption, and the `AdaptiveDeferral` "spend the
whole remaining budget" special case all inlined into the same loop body.

## Viable alternatives

- Keep budget as a bare `u32` passed by mutable reference through the scheduler.
  Rejected: refill-on-tick-advance and exhaustion semantics would stay untested in
  isolation, and the "deferral exhausts the entire remaining budget" special case
  (not just the request's own cost) would stay buried in a 300-line function.
- Model budget exhaustion as a `Result`-returning consumption only, with refill
  handled elsewhere. Rejected: refill-before-consume ordering is load-bearing (a tick
  advance must top up the budget before that tick's requests are evaluated against
  it) — splitting them invites an ordering bug at the call site.
- A `Budget` value object owning `cap`/`refill_per_tick`/`remaining`/`last_tick`, with
  `refill_through(tick)` and `try_consume(Weight) -> Result<(), BudgetExhausted>` as
  its only two mutators. Chosen.

## Decision

`Budget::try_consume` is the only failure mode (`BudgetExhausted`); the balance never
goes negative. `Budget::exhaust()` exists specifically for `AdaptiveDeferral`
(ADR-0026): a deferred request forces the remaining balance to zero rather than
deducting just its own cost, reproducing the base monolith's `budget = 0` line
exactly.

## Tradeoffs

Exposing `exhaust()` as a public method looks like an escape hatch from normal
consumption semantics. It is one, deliberately: it is the one documented place a
caller may bypass `try_consume`'s normal accounting, because the behavior it
reproduces is itself a deliberate, already-accepted deviation (ADR-0026).

## Consequences

`jitter-budget` has zero dependencies beyond `serde`/`thiserror` and is exercised by
every budget-gated admission strategy in `jitter-scheduler`.

## Validation

- `cargo test -p jitter-budget` (10 tests: consumption, exhaustion leaves balance
  unchanged, refill capping and idempotence, `ticks_until_available`, `exhaust`)
