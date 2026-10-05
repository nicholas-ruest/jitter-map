# ADR-0003: Retry amplification factor

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

Success rate alone hides recovery load. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- success only
- attempt count
- RAF plus outcome metrics

## Decision

We will make attempts/original requests the primary safety metric. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

RAF remains paired with success and latency. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `benchmark_gate`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
