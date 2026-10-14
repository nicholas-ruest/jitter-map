# Outcome memory

## Context

OutcomeRecord contains an eight-dimensional normalized embedding and deterministic metadata. RuVector is the only intelligence store. Local file backing is evaluation-only; production storage remains behind authorized RuVector/Cloud SQL. Aggregate roots protect invariants before state transitions; entities retain stable scenario identity; immutable value objects cross boundaries; domain events are append-only facts, never commands disguised as history. In the planned 22-crate map this is the Infrastructure Adapters bounded context's `jitter-adapter-ruvector` crate (docs/architecture.md §5, row 12), implementing the `MemorySink` port defined in `jitter-ports`; it is never a dependency of a domain crate.

## Language

OutcomeRecord, Embedding, RuVector, Cloud SQL, `MemorySink`, `RuVectorMemoryAdapter`. Outcome Record is the same ubiquitous-language term used across docs/ddd/01-ubiquitous-language.md; this context is its only authorized persistence path.

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
