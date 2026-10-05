# ADR-0015: Six-crate dependency direction

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

One package obscured domain and upstream boundaries. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- single crate
- microcrate explosion
- layered workspace

## Decision

We will separate domain, application, AWS, Ruvnet, evaluation, and CLI. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

more manifests, explicit ownership. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `cargo_metadata_direction`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
