# Context map

## Context

This document maps the same bounded contexts defined in docs/architecture.md §2 — it does
not invent new ones. The base commit (`3db7fd2`) implements six packages folding all of
these contexts into a monolithic domain/application split; docs/architecture.md and
ADR-0027 are the **planned** 22-crate target this map describes. Retry Modeling and
Admission supply scenarios and budget decisions to Simulation. AWS, RuVector, RVF, Ruflo,
and MetaHarness each sit behind their own adapter. Evidence seals outputs. Outcome Memory
stores summaries. Evaluation compares candidates. Application alone coordinates — it is
the only context permitted to call more than one adapter in the same use case. Aggregate
roots protect invariants before state transitions; entities retain stable scenario
identity; immutable value objects cross boundaries; domain events are append-only facts,
never commands disguised as history.

| Bounded context | Crates (planned, docs/architecture.md §5) |
|---|---|
| Domain Core | `jitter-scenario`, `jitter-workload`, `jitter-policy`, `jitter-budget`, `jitter-scheduler`, `jitter-outcome` |
| Domain Orchestration | `jitter-simulation`, `jitter-recovery` |
| Hexagonal Boundary | `jitter-ports`, `jitter-application` |
| Infrastructure Adapters | `jitter-adapter-aws`, `jitter-adapter-ruvector`, `jitter-adapter-rvf`, `jitter-adapter-ruflo`, `jitter-adapter-metaharness` |
| Cross-Cutting Support | `jitter-evidence`, `jitter-config`, `jitter-observability` |
| Composition Roots | `jitter-service`, `jitter-cli` |
| Fitness Evaluation | `jitter-evaluation` |
| Contract Verification | `jitter-contract-tests` (dev-only) |

## Language

Bounded Context, Context Map, Anti-Corruption Layer, Port, Adapter, Composition Root,
Upstream/Downstream relationship. This document reuses every domain term from
docs/ddd/01-ubiquitous-language.md unchanged; it adds only the DDD/hexagonal structural
vocabulary needed to describe how contexts relate, never a business term.

## Ownership and interfaces

The owning crate is linked from the traceability table (docs/traceability.md). Within the
planned map, `jitter-ports` is the only crate every adapter depends on for its contract;
`jitter-application` is the only crate permitted to depend on `jitter-ports` plus Domain
Orchestration; adapters are never a dependency of a domain, application, or ports crate.
`jitter-cli` and `jitter-service` are the only crates permitted to depend on everything,
because they alone are composition roots. No adapter type leaks into any Domain Core or
Domain Orchestration crate.

## Invariants

- Authority is always `none`.
- The same frozen input digest is used for every candidate.
- Invalid input fails before mutation.
- Persistence and evidence failures cannot become successful evaluations.
- Dependencies point inward only, toward Domain Core (docs/architecture.md §3); this is
  checked structurally, not by convention (docs/architecture.md §8, Gate 1).

## Domain events

The context emits explicit accepted, rejected, completed, cancelled, timed-out, or failed
facts as appropriate. IDs are content-derived where evidence identity matters.

## Failure and validation

Unit tests exercise local invariants; application contract tests exercise ports;
integration tests call real upstream APIs; the end-to-end CLI covers valid, invalid,
timeout, and persistence paths. Under the planned map, `jitter-contract-tests` is the
shared suite every `jitter-adapter-*` must invoke and pass (docs/architecture.md §5, row
22); it is a dev-dependency only and never appears in a runtime dependency graph.
