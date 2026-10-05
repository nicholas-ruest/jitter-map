# ADR-0017: Frozen trace comparison

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

Candidate-specific workloads enable benchmark gaming. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- random per candidate
- live only
- same trace and seed

## Decision

We will freeze scenario, capacity, seed, metric code, and thresholds. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

bounded evidence may miss other workloads. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `identical_input_digest`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
