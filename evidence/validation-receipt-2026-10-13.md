# Validation Receipt — 2026-10-13

## Binding

- Implementation commit: `0d76df41a86e51373bd06837a32e1339fc8611a7`
- Branch: `recovery/2026-10-11-jittermap-20-crate-design`
- Workdir: `/home/ruv/.local/share/dream-machine/worktrees/jitter-map-2026-10-11-design`

## Pinned Checker Provenance

- Factory commit: `80f006f6960bc4e491e97f51e890ff708e81aa88`
- Checker blob: `f3f8181a7aad5b57cbcf6a4a211a0c69b8b7b09e`
- Test blob: `36beb062ea6a360e90431b48536989734b61e184`
- Checker SHA256: `cd72ca58dcc38255ea48acb467293886c52ab8fb677d26c4463fa839b8e31250`

## Regression Tests (checker's own test suite)

- 3 passed, 0 failed

## Checker Execution

Exact checker command and exact stdout JSON (as observed):

```
node /home/ruv/.local/share/dream-machine/checkers/80f006f6960bc4e491e97f51e890ff708e81aa88/check-project-contract.mjs .
```

```json
{
  "status": "STRUCTURAL_PASS_REQUIRES_EVIDENCE_REVIEW",
  "counts": { "adrs": 49, "ddds": 12, "crates": 22, "animatedSvgs": 2 },
  "failures": [],
  "limitations": "Structural checks do not prove meaningful architecture, linked workspace membership, executed upstream integrations, benchmark validity, CI success, or publication authorization."
}
```

- Exit code: `0`

## Independently Observed Commands (exit 0 for all)

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` | exit 0 |
| `cargo test --locked --workspace --all-targets --all-features` | 107 tests passed, 0 failed |
| `cargo deny check` | advisories, bans, licenses, sources — all ok |
| `python3 scripts/check_dependency_direction.py` | GATE 1 PASSED — 22 workspace crates, dependency direction clean |
| `cargo run --locked` → `jitter-benchmark` | exit 0 |
| `cargo run --locked` → `jitter-fairness-benchmark` | exit 0 |
| `jitter-cli` run against `fixtures/fleet-retry-storm-22.json` | real run, exit 0 |

## Semantic Review

- 22 cargo packages
- 4595 Rust source lines
- Every library crate has at least one reverse workspace caller
- `jitter-cli` and `jitter-service` are executable roots
- Every crate has tests or supplies shared contract helpers
- Real executed dependencies confirmed: RuVector `ruvector-core` 2.3.1, RVF `rvf-crypto` 0.2.0, AWS `aws-smithy-types`

## Flywheel Composite Score

- Formula: `primary=(tenant_success_fairness or 0)*1000+successes`, `noopRate=1-(tenant_success_fairness or 0)`, `costPerWin=attempts/max(primary,1)`, `regressed=false`
- Anchor: `target/debug/jitter-benchmark`
- Holdout: `target/debug/jitter-fairness-benchmark`
- One promotion and one `verifyReplayBundle` pass — advisory only

## Darwin One-Task Suite

- Hash prefix: `4206e7c1695a`
- Verify hash: OK
- Task command: `npm test` — availability check only

## Unresolved Blockers

- No GitHub CI for this commit
- No PR or merge
- No repo readback of this commit
- No Gists
- Ruflo transport partial/unpinned
- MetaHarness project adapter has no live process round trip
- Independent reviewer promotion approval absent

## Status

`NONCOMPLIANT_BUILD` / `IN_PROGRESS`
