# ADR-0023: Darwin safety envelope

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

Unbounded search can overfit or weaken limits. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- no search
- free-form optimizer
- enumerated lattice

## Decision

We will vary only budget, refill, and costs inside fixed bounds. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

less novelty, no authority expansion. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `reject_out_of_envelope`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
