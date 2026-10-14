# ADR-0112: RuVector adapter isolation (`jitter-adapter-ruvector`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13
- Extends: ADR-0008

## Context

ADR-0027 planned `jitter-adapter-ruvector` as a `MemorySink`-only split out of the
base monolith's combined `jitter-adapter-ruvnet` package, which folded RuVector
memory and RVF witnessing into one crate and one embedding vector built from
`SimulationReport` fields.

## Viable alternatives

- Keep RuVector and RVF in one combined adapter. Rejected: this is the exact
  conflation ADR-0027 planned to split; `WitnessSink` and `MemorySink` are two
  separate ports with two separate contract-test suites, and one crate implementing
  both would hide which port each test is actually proving.
- Build the nearest-outcome embedding from `jitter-outcome::OutcomeSummary` directly.
  Rejected: `MemorySink::record` takes `&ReceiptRef` (ADR-0109), not a summary — the
  port boundary intentionally does not expose per-tenant vectors to the memory
  adapter.
- Build a 4-dimension embedding from `ReceiptRef`'s own numeric fields (retry
  amplification factor, p95 ticks, tenant success fairness, a constant), narrower
  than the base monolith's 8-dimension embedding (which had direct access to raw
  counts a `SimulationReport` carried but a `ReceiptRef` does not). Chosen.

## Decision

`RuVectorMemoryAdapter::open` wraps a real `ruvector_core::VectorDB` at a Cloud-SQL-
or local-redb-backed path. `record` searches for the 3 nearest prior outcomes by this
embedding, inserts the new one, and read-verifies the insert before returning.

## Tradeoffs

A narrower embedding is a real behavior change versus the base monolith — nearest-
outcome recall is now based on fewer signals. This follows directly from
`ReceiptRef`'s narrower DTO (ADR-0109) and is the traceable cost of that boundary
decision, not an independent regression.

## Consequences

A run still completes and seals a receipt without this adapter (the AWS/RuVector/RVF
degradation table in docs/specification.md already covers this ablation); this
adapter's own write path is scoped to the IAM-authorized RuVector/Cloud-SQL path only.

## Validation

- `cargo test -p jitter-adapter-ruvector` (2 tests: real vector round-trip/recall
  against a temp-directory-backed `VectorDB`, and the shared `MemorySink` contract
  suite)
