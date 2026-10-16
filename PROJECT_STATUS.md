# Project status

- **BUILD: BUILT.** The 22-crate composite system is published and validated. PR #15
  added the bounded Ruflo 3.56.10 registry and read-only public federation-sync
  process path; CI run 38838919923 passed the exact candidate tree
  `4e9176af1b519739a672a20d96f929802122e8d4`, which was squash-merged as
  `6a2fd3b15e6d2440733a4108f18e98586a94bfdb`. The pinned factory checker passed
  with 51 ADRs, 12 DDDs, 22 implemented crates, and 2 animated SVGs. Independent
  review verified real caller paths, executed AWS/RuVector/RVF/Ruflo/Darwin/Flywheel
  boundaries, frozen benchmarks and ablations, failure paths, README assets, CI,
  repository readback, and the owner-readback research/build Gists. Authority remains
  `none`; no policy was promoted or deployed.
- **Current implementation (published base plus current live-integration increment):** `cargo metadata` reports
  exactly 22 workspace packages — `jitter-scenario`, `jitter-workload`,
  `jitter-policy`, `jitter-budget`, `jitter-scheduler`, `jitter-outcome`,
  `jitter-simulation`, `jitter-recovery`, `jitter-ports`, `jitter-application`,
  `jitter-evidence`, `jitter-adapter-aws`, `jitter-adapter-ruvector`,
  `jitter-adapter-rvf`, `jitter-adapter-ruflo`, `jitter-adapter-metaharness`,
  `jitter-config`, `jitter-observability`, `jitter-service`, `jitter-cli`,
  `jitter-evaluation`, `jitter-contract-tests`. The base six-package implementation
  (`jitter-domain`, the old `jitter-application`/`jitter-adapter-aws`'s prior
  contract, `jitter-adapter-ruvnet`, the root `jitter-map` binary) was deleted as its
  replacement landed in each case, per docs/architecture.md §6's no-parallel-code
  rule — not left in place alongside the new crates.
- **Base SHA this migration started from:** `3db7fd2c7023e39ca81cd2ecad46c55e72d0cf70`
  (6 packages). **Design SHAs this migration implements:** `a818511` (start),
  `0eaa592` (canonical 22-crate system map, ADR-0027), `1afe4fc` (migration contract).
- **Gate 1 (structural), docs/architecture.md §8:** `python3
  scripts/check_dependency_direction.py` — a real, runnable `cargo metadata`-based
  checker, not a conceptual description — reports `GATE 1 PASSED: 22 workspace
  crates, dependency direction is clean.` It verifies no Domain Core/Orchestration
  crate depends on `jitter-ports`/any adapter/`jitter-config`/`jitter-observability`;
  no adapter appears in `jitter-application`'s or any domain crate's transitive
  graph; `jitter-contract-tests` appears only under `[dev-dependencies]`
  workspace-wide; and no dependency cycle exists among the 22 crates' normal edges.
- **Gate 2 (semantic), per docs/architecture.md §5:** `cargo test --workspace` passes
  with zero failures across every crate. Highlights: `jitter-simulation`'s
  `matches_base_monolith_golden_values_across_every_strategy` freezes a side-by-side
  verification against the deleted `jitter-domain` monolith across all six admission
  strategies (attempts/successes/terminal/denied/deferred all matched exactly, plus
  per-tenant vectors and `completed_at`, verified before deletion); `jitter-recovery`'s
  `interrupted_replay_matches_uninterrupted_digest_at_tick_six` and
  `interruption_deep_inside_outage_window_still_replays_identically` (tick 9) both
  reach the exact same `SimulationRun` as an uninterrupted run on a
  `fleet-retry-storm-22`-shaped scenario; `jitter-evaluation`'s
  `weighted_tenant_fair_outranks_unweighted_baseline_with_committed_delta` locks in a
  specific, independently re-derived fitness delta.
- **End-to-end verification against the real fixture
  (`fixtures/fleet-retry-storm-22.json`: 500 tenants, 3 at weight 20 / 497 at weight
  1, 12-tick outage, capacity 64/tick, horizon 400):** two automated `jitter-cli`
  integration tests spawn the real compiled binary as a subprocess —
  `fleet_retry_storm_22_weighted_matches_committed_golden_output`
  (`crates/jitter-cli/tests/golden_cli.rs`) asserts stdout matches a committed golden
  JSON byte-for-byte, and `interrupted_and_replayed_run_matches_uninterrupted_receipt`
  (`crates/jitter-cli/tests/recovery_cli.rs`) runs a plain baseline, an
  `--interrupt-after 6` capture, and a `--replay`, asserting the replayed receipt's
  stdout is identical to the uninterrupted baseline's. `jitter-service`'s `/run` and
  `/evaluate` endpoints are covered by black-box integration tests over a real
  `TcpListener`. Tracing output was moved to stderr (ADR-0118) specifically so these
  stdout comparisons stay clean.
- **Quality gates, this worktree:** `cargo build --workspace`, `cargo test
  --workspace` (0 failures across 49 test binaries), `cargo clippy --workspace
  --all-targets` (zero warnings), `cargo fmt --all -- --check` (clean), `cargo deny
  check` (advisories/bans/licenses/sources all `ok`) all pass as of this status.
- IMPLEMENTED (carried forward unchanged from base `3db7fd2`, now inside
  `jitter-scheduler`/`jitter-outcome`): opt-in `WeightedTenantFairBudget` plus the
  existing equal-share ablation, weighted workload validation, typed per-tenant
  outcomes, and Jain service fairness — reproduced bit-for-bit (see Gate 2 above).
- Documentation inventory: **51 ADRs** (0001–0027 plus 0101–0124) plus
  docs/adrs/index.md and docs/adrs/README.md; **12 DDD documents** plus
  docs/ddd/index.md and docs/ddd/README.md; one new structural-checker script
  (`scripts/check_dependency_direction.py`); one new committed fixture
  (`fixtures/fleet-retry-storm-22.json`).
- README: official versioned top-row marks (Rust, AWS SDK for Rust, RuVector/RVF,
  MetaHarness) and **2 project-specific animated SVGs** — `docs/assets/overview-hero.svg`
  and `docs/assets/execution-flow.svg` — remain present and unchanged.
- Authority: none. No evaluator or memory component may promote a policy — enforced
  at the type level (`jitter-evidence::Receipt.authority`,
  `jitter-evaluation::Fitness.authority` are both hardcoded `"none"`, with no mutator).
- **Known, honestly-carried limitations:** the Ruflo boundary is intentionally
  read-only and does not join or publish to a relay, deliver remote interruption
  authority, promote policies, or deploy services. Federation message bodies remain
  opaque and are not persisted. MetaHarness npm packages still omit `gitHead`, so
  registry versions and the inspected source pin remain separate provenance facts.
  `jitter-adapter-ruvector` uses a narrower 4-dimension outcome embedding than the
  historical monolith by ADR-0109. The skewed fairness candidates still trail the
  adaptive-drop success baseline; no superiority claim follows.

Frozen skewed workload (`8f5b79e0a345229171428559523a79a152b2f04a29d209e7ee68c40b40822bc7`),
measured on the base implementation this migration reproduces bit-for-bit (not a
re-roll on the new crates):

| Candidate | RAF | Successes | p95 | Denied | Jain fairness |
|---|---:|---:|---:|---:|---:|
| Adaptive drop | 3.7017 | 72 | 16 | 527 | 0.1250 |
| Equal-share tenant fair | 3.7100 | 63 | 15 | 536 | 0.8932 |
| Demand-weighted tenant fair | 3.7100 | 65 | 16 | 534 | 0.9684 |

Decision: **REVISE**. Demand weighting materially improves service fairness and recovers
two successes versus equal-share admission, but it still does not preserve the
adaptive-drop success baseline. It is not promoted. This conclusion is unchanged by the
migration — `jitter-evaluation`'s regression test now locks in the equivalent
comparison (`WeightedTenantFairBudget` vs. `TenantFairBudget` on `fairness_scenario`)
against the new 22-crate implementation.
