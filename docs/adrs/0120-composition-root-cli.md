# ADR-0120: Composition root — CLI (`jitter-cli`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13
- Extends: ADR-0024, ADR-0025

## Context

ADR-0027 planned `jitter-cli` to preserve the existing CLI JSON contract (ADR-0024)
while adding a `--replay <checkpoint>` flag. The base monolith's CLI (the root
`jitter-map` binary) read a scenario file, ran one policy, and printed an
`EvaluationReceipt` as JSON.

## Viable alternatives

- Keep the binary at the workspace root. Rejected: every other composition root and
  every other crate lives under `crates/`; a root-level binary would be the one
  exception to the 22-crate structure ADR-0027 planned.
- Drop the `--memory`/`--aws-max-attempts`/`--timeout-ms` flags in favor of requiring
  `jitter-config`'s environment variables directly. Rejected: this would break the
  existing CLI contract's flag-based ergonomics for no benefit — `jitter-config` can
  validate CLI-flag-derived values just as well as real environment variables.
- Move the binary into `crates/jitter-cli`, keep its `[[bin]] name = "jitter-map"` so
  the shipped binary name is unchanged, keep the existing flags, and build a synthetic
  environment map from them to pass through `jitter_config::load_from` before any
  adapter is constructed. Add `--interrupt-after <tick>` (captures and writes a
  checkpoint file, then exits) and `--replay <checkpoint>` (resumes from one) as new,
  additive flags. Chosen.

## Decision

Plain runs call `RunFleetRetryWorkload::execute_with_recovery` with a fresh
`RufloSignalAdapter` reporting no interruption — exercising `HarnessSink`'s real
caller edge even when recovery is not actually needed. `--interrupt-after` calls
`jitter_recovery::capture` directly and writes the resulting `Checkpoint` as JSON.
`--replay` reads that file back, regenerates the trace for a seed-consistency check,
and calls `jitter_recovery::replay`, finishing the result through
`RunFleetRetryWorkload::finish_run` — the same witness/evidence/memory pipeline a
plain run uses.

## Tradeoffs

None beyond what ADR-0024/ADR-0025 already accepted; this is an additive extension of
an already-frozen contract, not a revision of it.

## Consequences

`jitter-cli --scenario fixtures/fleet-retry-storm-22.json --policy
weighted-tenant-fair-budget` run end to end, then `--interrupt-after 6` followed by
`--replay`, produced a byte-identical `Receipt` (all three digests matched) to an
uninterrupted run — verified manually against the real composition root, not just
unit-tested in isolation.

## Validation

- `cargo build -p jitter-cli` (composition root, real executable)
- Manual end-to-end verification against `fixtures/fleet-retry-storm-22.json`:
  uninterrupted run, interrupt-after-6-then-replay run, and a fresh uninterrupted run
  in a separate memory directory all produced the identical `scenario_digest`/
  `outcome_digest`/`witness_digest` triple
