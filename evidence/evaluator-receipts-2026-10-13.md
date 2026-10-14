# Evaluator receipts — 2026-10-13

These receipts cover evaluator and coordination commands actually executed and
observed against this worktree today. They are distinct from the earlier base-
workspace Darwin/Flywheel run recorded in `evidence/darwin-output.txt`,
`evidence/darwin-winner.json`, `evidence/flywheel-output.json`, and
`evidence/flywheel-replay.json`, which measured the now-deleted six-package base
(`3db7fd2`) and are kept only as historical record.

## Darwin 0.10.3 — `bench create` / `bench verify`

- `bench create` wrote a one-task suite; recorded hash prefix `4206e7c1695a`.
- `bench verify` reported hash OK against that suite.
- The generated suite's task command is `npm test`, not this workspace's own
  `jitter-benchmark`/`jitter-fairness-benchmark` binaries. Because the task harness
  does not invoke a JitterMap policy at all, this result is **availability/provenance
  evidence only** — it shows Darwin 0.10.3 is reachable and produces a verifiable,
  hash-chained suite — and is **not candidate validation** of any retry-admission
  policy in this repository.

## Flywheel 0.1.12 — one live generation

- Executed one live generation using `target/debug/jitter-benchmark` as the frozen
  anchor and `target/debug/jitter-fairness-benchmark` as the holdout.
- Produced one candidate under an explicitly documented composite score and, under
  that score, promoted `AdaptiveBudget` to `WeightedTenantFairBudget`.
- `verifyReplayBundle` returned `pass`, with chain `gen1(strategy)` → `gen0(root)`.
- Per ADR-0010 (advisory authority) and ADR-0022 (promotion gate), **this evaluator
  result is advisory and is not promotion authority**. Flywheel's own "promoted"
  language describes its internal candidate-selection step, not a repository-level
  promotion of `WeightedTenantFairBudget` over `AdaptiveBudget` — that still requires
  an independent reviewer per docs/validation.md's "Independent semantic review"
  section, which has not happened.

## Ruflo Federation — identity call

- `mcp__codex_apps__ruflo_federation_federation_identity` read-only call succeeded.
- Identity value is omitted from this record.
- This is coordination/provenance availability evidence only. It confirms the Ruflo
  Federation relay is reachable; it is **not implementation evidence** for
  `jitter-adapter-ruflo`, whose real implementation status (a real, thread-safe
  `HarnessSink`/atomic signal primitive, with no live Ruflo network transport behind
  it) is tracked in PROJECT_STATUS.md and ADR-0114.
