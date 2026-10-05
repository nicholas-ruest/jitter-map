# ADR-0018: AWS standard baseline

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

A straw-man baseline is not enterprise prior art. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- custom imitation
- no baseline
- real RetryConfig

## Decision

We will construct and translate the real AWS standard config. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

not a claim to execute the full AWS client. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `aws_baseline`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
