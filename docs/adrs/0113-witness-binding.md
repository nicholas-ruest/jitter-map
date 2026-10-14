# ADR-0113: Witness binding (`jitter-adapter-rvf`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13
- Extends: ADR-0009

## Context

ADR-0027 planned `jitter-adapter-rvf` as a `WitnessSink`-only split out of the base
monolith's combined `jitter-adapter-ruvnet` package. See ADR-0112 for the shared
rationale on why the combined package was split in two.

## Viable alternatives

- Seal a `SimulationReport`'s full `Debug` representation, as the base monolith did.
  Rejected: `WitnessSink::write` takes `&OutcomeSummary` plus a scenario digest
  (ADR-0109), not a `SimulationReport` — that type no longer exists after
  `jitter-domain`'s deletion.
- Seal just the scenario digest, dropping the outcome entirely. Rejected: the witness
  would then attest to "this scenario ran" but not "with this outcome," which is a
  materially weaker claim than ADR-0009 commits to.
- Seal both the scenario digest and the outcome summary's `Debug` representation as
  two witness-chain entries, matching the base monolith's two-entry shape. Chosen.

## Decision

`RvfWitnessAdapter::write` builds two `WitnessEntry` records (scenario digest, then
outcome), chains them via `rvf_crypto::create_witness_chain`, verifies the chain, and
returns the chain's own SHAKE-256 digest as the `WitnessRef`.

## Tradeoffs

None beyond the type-shape change already covered by ADR-0109/ADR-0112.

## Consequences

An unsealed `OutcomeSummary` is still inspectable directly without this adapter (the
witness is an added integrity guarantee, not a precondition for the run to produce
useful output).

## Validation

- `cargo test -p jitter-adapter-rvf` (3 tests: real chain seal/verify, digest
  stability across repeated identical seals, and the shared `WitnessSink` contract
  suite — which also asserts the digest changes when the scenario digest changes)
