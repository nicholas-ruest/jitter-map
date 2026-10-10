# Admission context

## Purpose and model

BudgetState never becomes negative. `AdaptiveBudget` drops on exhaustion; `AdaptiveDeferral` reschedules until projected refill; `TenantFairBudget` uses the same drop budget but batches a tick into per-tenant FIFO queues and rotates round-robin admission. Weighted demand does not buy more shared budget. Aggregate and per-tenant outcomes expose the equality/throughput tradeoff. Every retry remains bounded by the horizon and `MAX_EVENTS`; this is not represented as a production SLA scheduler. Aggregate roots protect invariants before state transitions, and domain events are append-only facts.

## Ownership and interfaces

The owning crate is linked from the traceability table. Application ports specify required behavior, while AWS, RuVector, RVF, CLI, and evaluator code remain adapters. No adapter type leaks into `jitter-domain`.

## Invariants

- Authority is always `none`.
- The same frozen input digest is used for every candidate.
- Invalid input fails before mutation.
- Persistence and evidence failures cannot become successful evaluations.

## Domain events

The context emits explicit accepted, rejected, deferred, completed, cancelled, timed-out, or failed facts as appropriate. `deferred_retries` on `SimulationReport` makes the deferred fact typed and countable rather than inferred from attempt counts. IDs are content-derived where evidence identity matters.

## Failure and validation

Unit tests exercise local invariants; application contract tests exercise ports; integration tests call real upstream APIs; the end-to-end CLI covers valid, invalid, timeout, and persistence paths.
