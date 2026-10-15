# Validation

This document separates three things that are easy to conflate in status reporting:
evidence that already exists, the mechanical semantics of the automated checker, and the
independent human/semantic review the checker cannot replace. See
docs/implementation-contract.md for the exact crate-counting rule and
docs/architecture.md §8 for the two-gate process this document explains.

## Current evidence (base `3db7fd2`)

What is actually verified today, with its artifact:

- `cargo fmt --all -- --check`, `cargo clippy ... -D warnings`, `cargo test --workspace
  --all-targets --all-features`, `cargo deny check` — all exit 0 (last recorded in
  `evidence/executor-ledger-2026-10-10.md`).
- Real AWS RetryConfig translation, real RuVector insert/search/read-back, real RVF
  witness verification — recorded in docs/traceability.md and `evidence/cli-receipt*.json`.
- Frozen skewed-workload benchmark comparing adaptive-drop, equal-share, and
  demand-weighted tenant-fair admission — `evidence/fairness-benchmark.json`,
  PROJECT_STATUS.md.
- Darwin 0.10.3 / Flywheel 0.1.12 advisory evaluation, zero promotions, replay verified —
  `evidence/darwin-output.txt`, `evidence/flywheel-output.json`,
  `evidence/flywheel-replay.json`.
- This evidence covers the **former six-package base**, which no longer exists as a
  separate thing to compare against — `jitter-domain` and `jitter-adapter-ruvnet` were
  deleted once the 22-crate implementation below reproduced and superseded them. It is
  kept here as the historical record a migration ADR requires (ADR-0015's reversal
  rule), not as a description of the current repository state.

## Current evidence (22-crate implementation, this worktree, uncommitted)

- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -D warnings`,
  `cargo test --workspace` (49 test binaries, 0 failures), `cargo deny check` — all
  exit 0, re-run as the final step of this migration.
- `python3 scripts/check_dependency_direction.py` — a real, executable Gate 1 checker
  (not a conceptual description) — passes for all 22 crates, and is now wired into
  `.github/workflows/ci.yml` as its own step.
- Real AWS `RetryConfig` translation, real RuVector insert/search/read-back, real RVF
  witness verification, a real append-only MetaHarness evidence feed, and a real
  thread-safe Ruflo interruption-signal primitive — one per `jitter-adapter-*` crate,
  each gated by its own invocation of `jitter-contract-tests`.
- `jitter-simulation`'s golden-value regression test, verified side by side against
  the (now-deleted) base monolith before deletion; `jitter-recovery`'s
  interrupted-vs-uninterrupted digest match at tick 6 and tick 9;
  `jitter-evaluation`'s committed fitness-delta regression; `jitter-cli`'s golden-CLI
  and recovery-workflow integration tests, both spawning the real compiled binary.
- See docs/traceability.md for the per-crate test name mapped to each acceptance
  criterion, and docs/integration-matrix.md for the still-honest executed-vs-not split
  per external ingredient (Ruflo and MetaHarness in particular: a real port
  implementation exists, but no live external transport/process exists to round-trip
  with).

## Checker semantics (docs/architecture.md §8)

The structural checker is a necessary, not sufficient, condition for compliance. Its
semantics are mechanical and narrow by design:

- **Gate 1 — structural.** `scripts/check_dependency_direction.py` runs `cargo
  metadata` and fails with a nonzero exit if: a Domain Core/Orchestration crate
  depends on `jitter-ports`, any adapter, `jitter-config`, or `jitter-observability`;
  an adapter appears in `jitter-application`'s or any domain crate's transitive
  graph; `jitter-contract-tests` appears outside `[dev-dependencies]`; or a dependency
  cycle exists among normal (non-dev) edges. It proves the *shape* of the
  architecture, nothing about behavior. It is a real, runnable script, wired into
  `.github/workflows/ci.yml`, not a conceptual description awaiting one.
- **Gate 2 — semantic.** The per-crate acceptance criterion in docs/architecture.md §5:
  golden-digest comparisons, the fairness-regression replay, the §7
  interrupted-vs-uninterrupted digest match, and the §21 evaluation ranking. A crate
  passing Gate 1 with no Gate 2 result is "structurally staged," not complete.
- Gate 1 passing is mechanically checkable by any CI run; it cannot be gamed by a crate
  that has the right `Cargo.toml` edges but wrong behavior, because Gate 2 is required
  independently. Conversely Gate 2 cannot substitute for Gate 1 — a crate with correct
  output but a cyclic or inward-violating dependency graph is still non-compliant.

## Independent semantic review

The core-memory guidance this project operates under (docs/core-memory-guidance-map.md)
states two rules the checker cannot enforce mechanically, and which remain PENDING even
after the 22-crate implementation below, because no mechanism enforces them — only
convention:

- **Evaluators cannot self-promote.** No agent, evaluator, or memory component (Darwin,
  Flywheel, RuVector nearest-neighbor recall) may mark its own output as passing a gate.
  Promotion requires a reviewer independent of the component being evaluated
  (ADR-0010, ADR-0022). Today this is enforced only by convention (every evidence receipt
  in this repository carries `authority: none`); it has no independent-reviewer sign-off
  artifact yet.
- **Checker plus semantic review, both required.** An automated checker pass is
  necessary but not sufficient for merge; a separate semantic reviewer must also confirm
  the acceptance criterion's *meaning* was satisfied, not just its mechanical form (e.g.
  that a "golden digest match" test actually exercises `fleet-retry-storm-22` and not a
  degenerate empty scenario). No CI config or reviewer checklist implementing this exists
  yet for the planned crates — this is PENDING, consistent with
  docs/core-memory-guidance-map.md's row for this guidance.
- Until an independent-review mechanism is built and wired into CI, any claim that a
  planned crate is "semantically validated" must be treated as unverified and reported as
  such, not reported as done on the strength of the structural checker alone.

## Still not built, after the 22-crate implementation

Everything in this list was PENDING in the design-only version of this document. All
but one item below is now built; the remaining one is named because it genuinely
requires a party other than this migration's own authoring agent:

- ~~A CI job implementing Gate 1~~ — done: `.github/workflows/ci.yml` runs
  `scripts/check_dependency_direction.py` as its own step.
- ~~Per-crate Gate 2 test suites~~ — done: every crate has its own passing suite
  (docs/traceability.md maps each acceptance criterion to its test name).
- ~~The `jitter-contract-tests` shared suite~~ — done, and invoked by all 5 real
  adapters from their own dev-dependency test suites.
- ~~The kill-at-tick-6 interrupted-replay test and the `fleet-retry-storm-22`
  fixture~~ — done: `fixtures/fleet-retry-storm-22.json` is committed;
  `jitter-recovery`'s own test suite exercises tick 6 and tick 9.
- **Still not built: an independent-reviewer sign-off step**, distinct from the
  authoring agent, for every Gate-2 pass claimed above. This document, the 22 new
  ADRs, and every test in this migration were authored and run by the same bounded
  implementation agent that wrote the code they validate. That is exactly the kind of
  self-certification ADR-0010/ADR-0022's "evaluators cannot self-promote" principle
  warns against for *policy* promotion, and the same caution applies here: treat this
  migration's own "all gates pass" claim as a strong, mechanically-reproducible
  starting point for an independent reviewer, not as a substitute for one.

## ADR-0123 validation plan

The process adapters add a third gate after structure and local semantics:

1. Direct child-process fixtures validate working directory, timeout/kill,
   non-zero exit, malformed JSON, 64-KiB output cap, and receipt failures.
2. RuOS executes the installed pinned Ruflo CLI through the Rust adapter and reads
   back the schema-valid append-only receipt.
3. RuOS executes Darwin 0.10.3 suite verification and a Flywheel 0.1.12 bounded
   generation/replay through the Rust adapter and repository bridge.
4. Combined, Ruflo-disabled, and MetaHarness-disabled runs use the same frozen input
   and seed; their simulation digests must match while evidence availability differs.
5. `cargo fmt --all -- --check`, strict locked workspace Clippy, locked tests,
   dependency direction, `cargo deny check`, both benchmarks, and the pinned project
   structural checker must all pass against the exact candidate commit.

A ChatGPT MCP identity read, a CLI `--help`, a direct Node import, or a local contract
test is availability evidence only. None closes the adapter gate unless the Rust
process runner itself performs and validates the round-trip.
