# Operations and failure

## Purpose and model

States are validating, simulating, sealing, persisting, complete, cancelled, timed out, and failed. Metrics are RAF, success, p95, denied retries, budget use, and adapter latency; partial work is never graduated. Aggregate roots protect invariants before state transitions; entities retain stable scenario identity; immutable value objects cross boundaries; domain events are append-only facts, never commands disguised as history.

## Ownership and interfaces

The owning crate is linked from the traceability table. Application ports specify required behavior, while AWS, RuVector, RVF, CLI, and evaluator code remain adapters. No adapter type leaks into `jitter-domain`.

## Invariants

- Authority is always `none`.
- The same frozen input digest is used for every candidate.
- Invalid input fails before mutation.
- Persistence and evidence failures cannot become successful evaluations.

## Domain events

The context emits explicit accepted, rejected, completed, cancelled, timed-out, or failed facts as appropriate. IDs are content-derived where evidence identity matters.

## Failure and validation

Unit tests exercise local invariants; application contract tests exercise ports; integration tests call real upstream APIs; the end-to-end CLI covers valid, invalid, timeout, and persistence paths.
