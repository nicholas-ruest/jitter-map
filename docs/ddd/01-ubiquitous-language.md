# Ubiquitous language

## Context

This document is the canonical glossary context: every other bounded context (§2 of
docs/architecture.md) must use these terms unchanged, in Rust identifiers, JSON field
names, CLI help text, tests, and evidence. No bounded context may coin a synonym for a
term defined here. Aggregate roots protect invariants before state transitions; entities
retain stable scenario identity; immutable value objects cross boundaries; domain events
are append-only facts, never commands disguised as history.

## Language

Request, Attempt, Retry Admission, Retry Budget, Failure Class, Capacity Window, RAF,
Outcome Record, Witness Receipt, Candidate, and Promotion Eligibility. The planned
22-crate migration (ADR-0027, docs/architecture.md §5) adds no new business terms — it
renames implementation units (crates), not domain language. `Scenario`, `WorkloadTrace`,
`TickResult`, `Checkpoint`, `Receipt`, and `Fitness` are the typed Rust names for concepts
already named above (Retry Admission's input, its deterministic trace, its per-tick
outcome, its resumable state, its Witness Receipt, and its Promotion Eligibility score,
respectively); they are API vocabulary, not new ubiquitous-language terms.

## Ownership and interfaces

The owning crate is linked from the traceability table. Application ports specify required behavior, while AWS, RuVector, RVF, CLI, and evaluator code remain adapters. No adapter type leaks into any Domain Core crate (`jitter-scenario`, `jitter-workload`, `jitter-policy`, `jitter-budget`, `jitter-scheduler`, `jitter-outcome`).

## Invariants

- Authority is always `none`.
- The same frozen input digest is used for every candidate.
- Invalid input fails before mutation.
- Persistence and evidence failures cannot become successful evaluations.

## Domain events

The context emits explicit accepted, rejected, completed, cancelled, timed-out, or failed facts as appropriate. IDs are content-derived where evidence identity matters.

## Failure and validation

Unit tests exercise local invariants; application contract tests exercise ports; integration tests call real upstream APIs; the end-to-end CLI covers valid, invalid, timeout, and persistence paths.
