# ADR-0025: Conservative repository recovery

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

The old repository is useful but not factory-complete. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- abandon
- cosmetic patch
- evolutionary recovery

## Decision

We will retain the name while replacing the single-client thesis before 1.0. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

breaking CLI change is explicit and traceable. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `tree_inventory`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
