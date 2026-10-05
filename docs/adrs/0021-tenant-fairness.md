# ADR-0021: Tenant fairness

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

A shared budget can be monopolized by event order. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- FIFO
- weighted queue
- stable tenant rounds

## Decision

We will order same-tick retries by deterministic tenant rounds. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

not a full SLA scheduler, avoids obvious starvation. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `fairness_fixture`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
