# Evidence context

## Context

Receipts bind scenario and policy digests, upstream revisions, seed, environment, commands, statuses, output digest, RVF root, and authority none. Corrections append; they do not rewrite. Aggregate roots protect invariants before state transitions; entities retain stable scenario identity; immutable value objects cross boundaries; domain events are append-only facts, never commands disguised as history. In the planned 22-crate map this is the Cross-Cutting Support bounded context's `jitter-evidence` crate (docs/architecture.md §5, row 16), consuming `WitnessRef` from `jitter-adapter-rvf` and `OutcomeSummary` from `jitter-outcome`, never depended on by a domain crate.

## Language

Receipt, Witness Root, Scenario Digest, Authority None, `WitnessRef`, `Receipt::bind`. "Witness Receipt" in docs/ddd/01-ubiquitous-language.md is this context's bound output; the raw witness artifact itself belongs to docs/ddd/09-upstream-integrations.md's RVF adapter boundary.

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
