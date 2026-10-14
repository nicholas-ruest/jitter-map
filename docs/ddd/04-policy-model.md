# Policy aggregate

## Context

RetryPolicy owns attempt cap, delay range, jitter behavior, retryable classes, costs, budget, refill, and upstream revision. `PolicyKind` selects the admission strategy: `LocalJitter` and `AwsStandard` are not budget-gated; `AdaptiveBudget` and `AdaptiveDeferral` share the same budget fields but differ only in what happens when the budget is temporarily insufficient, per ADR-0026. Adapter provenance is data, never hidden global state. Aggregate roots protect invariants before state transitions; entities retain stable scenario identity; immutable value objects cross boundaries; domain events are append-only facts, never commands disguised as history. In the planned 22-crate map this is the Domain Core bounded context's `jitter-policy` crate (docs/architecture.md §5, row 3); `DemandWeighted` is the planned `RetryPolicy` impl that must reproduce the current base's `WeightedTenantFairBudget` output bit-for-bit.

## Language

RetryPolicy, PolicyKind, LocalJitter, AwsStandard, AdaptiveBudget, AdaptiveDeferral, TenantFairBudget, WeightedTenantFairBudget, `RetryPolicy` trait, `Decision`. These name policy strategies only; Retry Budget itself is owned by docs/ddd/06-admission-budget.md, not redefined here.

## Ownership and interfaces

The owning crate is linked from the traceability table. Application ports specify required behavior, while AWS, RuVector, RVF, CLI, and evaluator code remain adapters. No adapter type leaks into any Domain Core crate (`jitter-scenario`, `jitter-workload`, `jitter-policy`, `jitter-budget`, `jitter-scheduler`, `jitter-outcome`).

## Invariants

- Authority is always `none`.
- The same frozen input digest is used for every candidate.
- Invalid input fails before mutation.
- Persistence and evidence failures cannot become successful evaluations.

## Domain events

The context emits explicit accepted, rejected, completed, cancelled, timed-out, or failed facts as appropriate. IDs are content-derived where evidence identity matters.

## Failure and validation

Unit tests exercise local invariants; application contract tests exercise ports; integration tests call real upstream APIs; the end-to-end CLI covers valid, invalid, timeout, and persistence paths.
