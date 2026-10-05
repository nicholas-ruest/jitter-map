# ADR-0010: Advisory authority

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

Evaluation must not silently alter production behavior. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- auto apply
- emit deployment
- authority none

## Decision

We will all results carry authority none and require an external owner. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

no automatic promotion even for a clear winner. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `authority_schema`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
