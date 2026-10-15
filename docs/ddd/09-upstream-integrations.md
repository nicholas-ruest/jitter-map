# Upstream integration

## Context

AWS RetryConfig, RuVector VectorDB, RVF crypto, Ruflo provenance, Darwin proposals, and Flywheel replay each sit behind a typed anti-corruption layer with timeouts and exact provenance. Aggregate roots protect invariants before state transitions; entities retain stable scenario identity; immutable value objects cross boundaries; domain events are append-only facts, never commands disguised as history. In the planned 22-crate map this context spans the five Infrastructure Adapter crates in docs/architecture.md §5 (rows 11-15: `jitter-adapter-aws`, `jitter-adapter-ruvector`, `jitter-adapter-rvf`, `jitter-adapter-ruflo`, `jitter-adapter-metaharness`) plus the `jitter-ports` trait definitions (row 9) they implement; see docs/integration-matrix.md for executed-vs-planned status and pins per ingredient.

## Language

RetryConfig, VectorDB, Witness Chain, Ruflo Provenance, Darwin Proposal, Flywheel Replay, `RetryConfigSource`, `WitnessSink`, `MemorySink`, `HarnessSink`, `FlywheelSink`. These are port/adapter vocabulary; the business terms they carry (Retry Budget, Witness Receipt, Outcome Record, Candidate) remain owned by their respective contexts above.

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

## Live process aggregate

`ProcessEvaluation` is the aggregate root. `ProcessInvocation` is an entity identified
by an input digest. `ExecutablePath`, `ArgvFingerprint`, `OutputDigest`, `Timeout`, and
`Authority` are immutable value objects. The aggregate accepts a completion only when
the executable was canonicalized, the fixed command exited zero within the timeout,
output stayed within the cap, the typed schema validated, and the append-only receipt
was durably written.

The Ruflo and MetaHarness adapters own their process anti-corruption layers. Domain and
application crates see only `RufloProvenance`, `EvaluationReplay`, and `PortError`.
Events are `ProcessStarted`, `ProcessCompleted`, `ProcessTimedOut`, `ProcessRejected`,
and `ReceiptAppended`; events never grant promotion authority.

Failure is fail-closed at the evidence boundary: a failed external process cannot
produce a success receipt or promote a candidate. The simulation remains available in
the documented ablations, preserving degraded-mode operation without pretending the
missing evidence exists.
