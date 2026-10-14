# ADR-0109: Ports and adapters boundary (`jitter-ports`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-ports` as trait-only outbound port definitions — zero
implementations — with `RetryConfigSource`, `WitnessSink`, `MemorySink`, `HarnessSink`,
and `FlywheelSink`. The planning table's shown signatures for `WitnessSink::write` and
`MemorySink::record` take `&SimulationRun` and `&RunReceipt` respectively.

## Viable alternatives

- Implement the five port signatures exactly as shown in the planning table,
  depending on `jitter-simulation` (for `SimulationRun`) and `jitter-evidence` (for
  `Receipt`). Rejected: docs/architecture.md §3's dependency diagram draws no arrow
  from `PORTS` to `SIM` or `EVID`, and that absence is structurally checked (Gate 1) —
  naming either crate here would be exactly the inward-dependency violation the
  checker exists to catch.
- Depend on nothing and pass raw primitives (strings, numbers) through every port
  signature. Rejected: this would make every adapter's contract-test suite
  (`jitter-contract-tests`) untyped and unable to assert anything meaningful about
  shape.
- Depend only on `jitter-outcome` (already required for `OutcomeEvent` on
  `HarnessSink`) and define small port-local DTOs (`WitnessRef`, `ReceiptRef`,
  `RetryBudgetSpec`, `InterruptionSignal`, `CandidateResult`) for everything else.
  Chosen, documented as a deviation in the crate's own doc comment.

## Decision

Five traits, all `Result<_, PortError>`-returning, all taking the narrowest DTO that
still lets `jitter-contract-tests` assert something real: `RetryConfigSource::
retry_budget`, `WitnessSink::write(&OutcomeSummary, &str)`, `MemorySink::
record(&ReceiptRef)`, `HarnessSink::{on_tick, poll_interruption}`, `FlywheelSink::
submit(&CandidateResult)`. `jitter-application`, which already depends on
`jitter-ports`, `jitter-simulation`, and `jitter-evidence` together, is responsible
for narrowing a real `Receipt`/`SimulationRun` down to the DTO before calling through
a port.

An initially-cyclic design — `jitter-ports` depending on `jitter-contract-tests` as a
dev-dependency so its own mocks could invoke the shared contract-assertion functions —
was tried and reverted: Cargo's dev-dependency-cycle support duplicates the pivot
crate's compilation unit, so the mock's trait impls and the contract function's trait
bound resolved to two different compiled instances of the same trait and failed to
type-check. The mocks are instead a normal (not `#[cfg(test)]`-gated) `pub mod mocks`
module, exercised directly by `jitter-ports`' own tests and reused — unmodified — by
`jitter-application`'s mocked end-to-end test.

## Tradeoffs

Port-local DTOs duplicate a handful of fields that also exist on `jitter-evidence::
Receipt` (digests, authority, fairness/RAF numbers). The duplication is the price of
keeping the dependency graph's one truly load-bearing rule — nothing in Domain Core or
the Hexagonal Boundary ever points outward — mechanically checkable rather than
aspirational.

## Consequences

Every `jitter-adapter-*` depends on `jitter-ports` to implement a trait; no adapter is
a dependency of `jitter-ports`, `jitter-application`, or any domain crate — verified by
`scripts/check_dependency_direction.py`.

## Validation

- `cargo test -p jitter-ports` (5 tests exercising the five mock implementations
  directly)
- Every one of the five real adapters (`jitter-adapter-aws`, `-rvf`, `-ruvector`,
  `-ruflo`, `-metaharness`) invokes and passes the matching `jitter-contract-tests`
  assertion function from its own dev-dependency test suite
