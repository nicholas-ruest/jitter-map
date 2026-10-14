# Simulation context

## Context

The engine owns an ordered event heap and deterministic RNG. Ordering is time, tenant round, request ID, attempt. It emits completed, cancelled, deadline, and limit outcomes without sleeping. Aggregate roots protect invariants before state transitions; entities retain stable scenario identity; immutable value objects cross boundaries; domain events are append-only facts, never commands disguised as history. In the planned 22-crate map this is the Domain Orchestration bounded context's `jitter-simulation` crate (docs/architecture.md §5, row 7), composed only from the Domain Core crates (`jitter-scenario`, `jitter-workload`, `jitter-policy`, `jitter-budget`, `jitter-scheduler`, `jitter-outcome`); `jitter-recovery` (row 8) extends it with checkpoint replay (§7).

## Language

Event Heap, Tick, Deterministic RNG, `SimulationRun`, `Checkpoint`. "Outcome" here means the same typed `OutcomeEvent` owned by docs/ddd/06-admission-budget.md and docs/ddd/01-ubiquitous-language.md's Outcome Record — this context produces it, it does not redefine it.

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
