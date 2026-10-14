# Security and policy

## Context

Inputs are untrusted and bounded. The service accepts no secrets, owns no deployment authority, uses safe configured paths, and prevents evaluators or child work from widening its SafetyEnvelope. Aggregate roots protect invariants before state transitions; entities retain stable scenario identity; immutable value objects cross boundaries; domain events are append-only facts, never commands disguised as history. In the planned 22-crate map this context's constraints bind the Composition Root crates (`jitter-cli`, `jitter-service`, docs/architecture.md §5 rows 19-20) and `jitter-config` (row 17), which is the only crate permitted to load untrusted configuration before any adapter is constructed.

## Language

SafetyEnvelope, Untrusted Input, Advisory Authority, `AppConfig`, `ConfigError`. "Authority: none" (docs/ddd/01-ubiquitous-language.md's Promotion Eligibility) is enforced here as a security invariant, not merely a reporting convention.

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
