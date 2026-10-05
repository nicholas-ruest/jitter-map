# ADR-0022: Promotion gate

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

A score must not become policy authority. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- minimum RAF wins
- narrative only
- multi-metric eligibility

## Decision

We will gate RAF, success, latency, fairness, and evidence integrity. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

no winner is a valid result. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `gate_never_promotes`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
