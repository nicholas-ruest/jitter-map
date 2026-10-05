# ADR-0019: Ruvnet-only ablation

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

Composition needs evidence for each ingredient. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- combined only
- feature flag
- explicit ablation

## Decision

We will run local policy with the same Ruvnet evidence path. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

more cases but isolates AWS contribution. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `ablation_rows`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
