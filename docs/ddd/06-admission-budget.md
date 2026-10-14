# Admission context

## Context

BudgetState never becomes negative. `AdaptiveBudget` drops on exhaustion; `AdaptiveDeferral` reschedules until projected refill; `TenantFairBudget` uses the same drop budget but batches a tick into per-tenant FIFO queues and rotates round-robin admission. Weighted demand does not buy more shared budget. Aggregate and per-tenant outcomes expose the equality/throughput tradeoff. Every retry remains bounded by the horizon and `MAX_EVENTS`; this is not represented as a production SLA scheduler. Aggregate roots protect invariants before state transitions, and domain events are append-only facts. In the planned 22-crate map this context splits across two Domain Core crates: `jitter-budget` (docs/architecture.md §5, row 4, quantum/exhaustion bookkeeping) and `jitter-scheduler` (row 5, same-tick admission ordering over `TenantQueues`); both are invoked from `jitter-simulation`.

## Language

BudgetState, Quantum, FIFO Round, Starvation, Jain Fairness, `TenantQueues`, `Budget::try_consume`, `TickResult`. Retry Budget itself (the ubiquitous-language term) is this context's aggregate root; `AdaptiveBudget`/`AdaptiveDeferral`/`TenantFairBudget`/`WeightedTenantFairBudget` are policy *strategies* owned by docs/ddd/04-policy-model.md, not by this context.

## Ownership and interfaces

The owning crate is linked from the traceability table. Application ports specify required behavior, while AWS, RuVector, RVF, CLI, and evaluator code remain adapters. No adapter type leaks into any Domain Core crate (`jitter-scenario`, `jitter-workload`, `jitter-policy`, `jitter-budget`, `jitter-scheduler`, `jitter-outcome`).

## Invariants

- Authority is always `none`.
- The same frozen input digest is used for every candidate.
- Invalid input fails before mutation.
- Persistence and evidence failures cannot become successful evaluations.

## Domain events

The context emits explicit accepted, rejected, deferred, completed, cancelled, timed-out, or failed facts as appropriate. `deferred_retries` on `SimulationReport` makes the deferred fact typed and countable rather than inferred from attempt counts. IDs are content-derived where evidence identity matters.

## Failure and validation

Unit tests exercise local invariants; application contract tests exercise ports; integration tests call real upstream APIs; the end-to-end CLI covers valid, invalid, timeout, and persistence paths.

## Proportional admission

The Admission Budget context now exposes two fair schedulers: equal-share round robin and demand-weighted round robin. Both own only queue order; the shared budget continues to own retry admission. Evaluation must compare both schedulers with the throughput-first adaptive baseline and may not promote either.
