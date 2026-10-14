# Application workflows

## Context

The current base's `evaluate` workflow validates, resolves policy, simulates, seals,
appends, searches, and returns a receipt; any sealing or persistence failure yields an
incomplete error and cannot produce promotion eligibility. In the planned 22-crate map
(docs/architecture.md §4-§5) this workflow is reshaped, not replaced, into
`jitter-application`'s `RunFleetRetryWorkload` use case (row 10): it is generic over the
`jitter-ports` traits (`RetryConfigSource`, `WitnessSink`, `MemorySink`) and is invoked
only by the two composition roots, `jitter-cli` and `jitter-service` (rows 19-20), which
inject the concrete adapters. `jitter-application` must never name a concrete adapter
crate directly — dependency injection happens through generics or trait objects supplied
by the composition root. The planned end-to-end exercise of this context is the
`fleet-retry-storm-22` workload (docs/architecture.md §4) plus its interrupted-replay
variant (§7), both routed through this same use case. Aggregate roots protect invariants
before state transitions; entities retain stable scenario identity; immutable value
objects cross boundaries; domain events are append-only facts, never commands disguised
as history.

## Language

Use Case, Port, Composition Root, Dependency Injection, `RunFleetRetryWorkload`,
`AppError`, `RunReceipt`. "Evaluate" (the current base's workflow name) and
`RunFleetRetryWorkload` (the planned name) refer to the same ubiquitous-language concept;
renaming a crate's entrypoint is not a new domain term and must not be tracked as one.

## Ownership and interfaces

The owning crate is linked from the traceability table (docs/traceability.md): today
`jitter-application`; under the planned map, still `jitter-application`, generic over
`jitter-ports`. Application ports specify required behavior, while AWS, RuVector, RVF,
Ruflo, MetaHarness, CLI, and evaluator code remain adapters or composition roots. No
adapter type leaks into any Domain Core crate (`jitter-scenario`, `jitter-workload`,
`jitter-policy`, `jitter-budget`, `jitter-scheduler`, `jitter-outcome`) or Domain
Orchestration crate (`jitter-simulation`, `jitter-recovery`).

## Invariants

- Authority is always `none`.
- The same frozen input digest is used for every candidate.
- Invalid input fails before mutation.
- Persistence and evidence failures cannot become successful evaluations.
- `jitter-application`'s dependency graph contains zero `jitter-adapter-*` crates
  (docs/architecture.md §8, Gate 1); this is checked, not assumed.

## Domain events

The context emits explicit accepted, rejected, completed, cancelled, timed-out, or failed
facts as appropriate. IDs are content-derived where evidence identity matters.

## Failure and validation

Unit tests exercise local invariants; application contract tests exercise ports against
`jitter-ports` mocks before any real adapter exists (docs/architecture.md §6, Phase 3);
integration tests call real upstream APIs; the end-to-end CLI covers valid, invalid,
timeout, and persistence paths. The planned acceptance test for this context is a mocked
end-to-end run of `fleet-retry-storm-22` through `RunFleetRetryWorkload` using only
`jitter-ports` mocks, proving the application layer needs no concrete adapter to be
correct.
