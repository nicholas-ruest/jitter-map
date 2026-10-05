# ADR-0008: RuVector outcome memory

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

Policy selection needs prior outcomes without a service-owned database. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- JSON files
- SQLite
- RuVector

## Decision

We will persist searchable outcome embeddings through ruvector-core. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

similarity is advisory and raw receipts stay canonical. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `vector_round_trip`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
