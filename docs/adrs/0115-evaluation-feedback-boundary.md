# ADR-0115: Evaluation feedback boundary (`jitter-adapter-metaharness`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13
- Extends: ADR-0023

## Context

ADR-0027 planned `jitter-adapter-metaharness` to bridge `jitter-evaluation`'s scoring
to the Darwin/Flywheel feedback loop via `FlywheelSink`. docs/integration-matrix.md
records a provenance gap for this ingredient: the inspected MetaHarness source commit
has no corresponding npm `gitHead`, so the pin is source-commit evidence only.

## Viable alternatives

- Call a MetaHarness binary or network service directly. Rejected: no such service is
  reachable from this workspace's test environment, and the existing base-monolith
  equivalent (`jitter-candidate`) was itself a subprocess *called by* MetaHarness, not
  a client *calling into* it — there is no "submit" direction to call today.
- Submit silently to `/dev/null` (a stub that always returns `Ok`). Rejected: would
  satisfy the trait but produce no inspectable evidence, violating ADR-0014's
  append-only evidence requirement.
- Append every submitted `CandidateResult` as a JSON-Lines record to a feed file,
  which a real Flywheel replay process can tail-read as its evaluation feedback
  stream. Chosen.

## Decision

`MetaharnessAdapter::submit` opens its configured feed path in append mode and writes
one JSON line per candidate result. Existing lines are never rewritten, matching
ADR-0014.

## Tradeoffs

This adapter proves the `FlywheelSink` contract and produces real, inspectable,
append-only evidence, but does not prove a live round-trip with an actual
Darwin/Flywheel process reading the feed — carrying forward the same honest
provenance gap docs/integration-matrix.md already records for this ingredient.

## Consequences

Darwin/Flywheel evaluation remains advisory-only; a run completes with or without this
adapter (ADR-0023's safety envelope already assumes this independence).

## Validation

- `cargo test -p jitter-adapter-metaharness` (3 tests: JSON-Lines append, existing
  lines never rewritten, the shared `FlywheelSink` contract suite)
- `jitter-service`'s `/evaluate` endpoint is a real caller, submitting a scored
  candidate and asserting it outranks the unweighted baseline
