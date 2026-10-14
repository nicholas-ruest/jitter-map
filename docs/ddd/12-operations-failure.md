# Operations and failure

## Context

States are validating, simulating, sealing, persisting, complete, cancelled, timed out, and failed. Metrics are RAF, success, p95, denied retries, budget use, and adapter latency; partial work is never graduated. Aggregate roots protect invariants before state transitions; entities retain stable scenario identity; immutable value objects cross boundaries; domain events are append-only facts, never commands disguised as history. In the planned 22-crate map this context's failure/replay path is owned jointly by `jitter-recovery` (docs/architecture.md §5, row 8) and `jitter-adapter-ruflo` (row 14); the kill-at-tick-6-and-resume scenario in §7 is this context's concrete acceptance test.

## Language

State (validating/simulating/sealing/persisting/complete/cancelled/timed out/failed), Metric, Partial Work, `Checkpoint`, interruption signal. "Partial work is never graduated" is this context's restatement of the Invariants section's evaluation-failure rule below.

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
