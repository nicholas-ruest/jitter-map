# ADR-0001: System-level retry admission

- Status: Accepted for bounded MVP
- Date: 2026-10-04

## Context

Retries are interacting load rather than independent client behavior. The decision must fit a deterministic Rust vertical slice, exact-source evidence, and advisory-only governance.

## Viable alternatives

- keep the local calculator
- rely on SDK defaults
- use a shared admission budget

## Decision

We will meter retry cost by failure class from a scenario-scoped budget. The public contract remains typed, bounded, and explicit about unsupported behavior.

## Rationale and tradeoffs

system safety is visible, at the cost of a coordination dependency. This is preferred because it preserves replayability and makes the operational risk measurable instead of hiding it in an adapter or evaluator.

## Consequences

The owning crate and its callers must preserve this invariant. A reversal requires a new ADR, migration note, and replay of the frozen corpus. Darwin, Flywheel, and memory systems may propose or score an alternative but cannot promote it.

## Validation

The executable check is `shared_budget_limits_amplification`. CI and the evidence receipt bind its result to an exact commit; a missing live integration is reported as blocked rather than mocked as passing.
