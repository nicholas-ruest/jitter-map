# ADR-0026: Adaptive deferral admission

- Status: Accepted for bounded MVP
- Date: 2026-10-09

## Context

ADR-0006 debits a shared budget before admitting each retry and converts an exhausted budget directly into a terminal failure. *Retry Amplification in Distributed Systems* (arXiv:2608.25403) motivates shared-budget admission but does not require that temporary exhaustion be treated as permanent. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance, and it must not change ADR-0006's existing behavior.

## Viable alternatives

- keep only immediate drop-on-exhaustion (status quo ADR-0006)
- unconditionally defer every exhausted retry regardless of refill feasibility
- deterministically defer only when projected refill lands within the horizon, otherwise fail closed

## Decision

We add `PolicyKind::AdaptiveDeferral` as a distinct policy, evaluated alongside the unchanged `AdaptiveBudget` ablation. When the shared budget is below the failure cost, the engine computes the number of ticks until projected refill (`ceil((cost - budget) / refill_per_tick)`) and reschedules the retry at `tick + wait` instead of terminating it, provided `refill_per_tick > 0` and `tick + wait <= horizon_ticks`. Otherwise it fails closed into the same denied-and-terminal path as `AdaptiveBudget`. The reservation clamps budget to zero rather than going negative; it never leaks the deferred retry past `MAX_EVENTS` or the scenario horizon, both of which are enforced by the existing event loop. The typed `SimulationReport` carries a new `deferred_retries: u64` counter so deferral is observable evidence, not an inferred side effect. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

Deferral can convert a would-be denial into an eventual success once the fleet-level budget recovers, but it also lets a request spend more of its fixed `max_attempts` ceiling waiting on budget instead of failing immediately, and the zero-clamped reservation is optimistic under contention from other tenants. On the frozen fixture this measured worse on success count and RAF than the drop ablation (see the benchmark table in README.md and `evidence/benchmark.json`), which is reported as-is rather than adjusted to look favorable. This is preferred over unconditional deferral because it preserves replayability, keeps the budget invariant (`BudgetState` never negative) intact, and makes the operational risk of waiting-versus-dropping measurable instead of hiding it in an adapter or evaluator.

## Consequences

`AdaptiveBudget` is unchanged bit-for-bit in behavior; `AdaptiveDeferral` is additive and does not alter existing call sites that do not opt into it. The owning crate and its callers must preserve the invariant that a denied retry never re-enters the event queue, while a deferred retry re-enters it only inside the bounded horizon. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable checks are `deferral_recovers_denied_budget`, `deferral_fails_closed_without_refill`, `deferral_replay_is_stable`, and `deferral_fails_closed_beyond_horizon` in `crates/jitter-domain/src/lib.rs`. CI and the evidence receipt bind their result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
