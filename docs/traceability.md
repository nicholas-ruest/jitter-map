# Traceability

## Historical base (`3db7fd2`, superseded by the 22-crate implementation below)

This table describes the deleted six-package workspace (`jitter-domain`,
`jitter-adapter-ruvnet`, the root `jitter-map` binary, and the rest). It is kept as
the historical record ADR-0015's reversal rule requires, not as a description of any
crate that currently exists in this repository.

| Criterion | Context | Crate | ADRs | Tests |
|---|---|---|---|---|
| Deterministic replay | Simulation | jitter-domain | 0002, 0007 | replay_is_stable |
| Fleet admission | Admission | jitter-domain | 0001, 0006, 0021 | budget_limits_amplification, weighted_assignment_and_fair_replay_are_deterministic, invalid_weight_boundary_fails_closed |
| Adaptive deferral | Admission | jitter-domain | 0006, 0026 | deferral_recovers_denied_budget, deferral_fails_closed_without_refill, deferral_replay_is_stable, deferral_fails_closed_beyond_horizon |
| AWS truth | Integration | jitter-adapter-aws | 0004, 0005, 0018 | real_retry_config_translates |
| Auditable memory | Evidence | jitter-adapter-ruvnet | 0008, 0009, 0014 | real_vector_round_trip_and_witness |
| Frozen comparison | Evaluation | jitter-evaluation | 0003, 0017, 0019, 0022 | candidates_share_digest_and_never_promote |
| CLI slice | Workflow | jitter-map | 0024, 0025 | workflow invocation |

| Proportional service fairness | Scenario / Admission Budget / Evaluation | jitter-domain, jitter-evaluation, jitter-map | ADR-0021 | `weighted_policy_tracks_demand_and_improves_service_fairness`; `jitter-fairness-benchmark` | `evidence/fairness-benchmark.json` |

## 22-crate implementation (docs/architecture.md §5, ADR-0027 + ADR-0101–ADR-0122 — implemented, this worktree)

Every row below is now implemented and passing in this worktree (PROJECT_STATUS.md);
"Tests/evidence" names the test that actually satisfies it today, not a test that
must still be written. `jitter-domain` and `jitter-adapter-ruvnet`, which this table
previously treated as "current base," were deleted as their replacements landed
(docs/architecture.md §6) — the "Current base" table above now describes history, not
the present state of the repository.

| Acceptance criterion | Context | Crate | ADR | DDD doc | Tests/evidence |
|---|---|---|---|---|---|
| Scenario invariants hold with zero non-domain deps | Domain Core | `jitter-scenario` | ADR-0101 | docs/ddd/03-scenario-model.md | `cargo test -p jitter-scenario` (12 tests); `scripts/check_dependency_direction.py` |
| Deterministic trace generation, byte-identical across runs | Domain Core | `jitter-workload` | ADR-0102 | docs/ddd/03-scenario-model.md | `determinism_same_seed_same_trace_bytes` |
| `DemandWeighted` policy reproduces base output bit-for-bit | Domain Core | `jitter-policy` | ADR-0103; extends ADR-0021 | docs/ddd/04-policy-model.md | 6 decision tests; exercised bit-for-bit via `jitter-simulation`'s golden-value test |
| Budget consumption/refill/exhaustion matches base reference exactly | Domain Core | `jitter-budget` | ADR-0104 | docs/ddd/06-admission-budget.md | 10 unit tests incl. exhaustion, refill capping/idempotence, deferral `exhaust()` |
| Zero starvation, matches golden fairness trace | Domain Core | `jitter-scheduler` | ADR-0105 | docs/ddd/06-admission-budget.md | 8 tests incl. fair/weighted ordering; exercised via `jitter-simulation`'s regression |
| `OutcomeSummary` JSON digest stable across repeated runs | Domain Core | `jitter-outcome` | ADR-0106 | docs/ddd/01-ubiquitous-language.md (Outcome Record) | 11 tests incl. `serde_round_trip_is_exact` |
| Zero non-domain deps; deterministic full-stack E2E hash | Domain Orchestration | `jitter-simulation` | ADR-0107 | docs/ddd/05-simulation-engine.md | `matches_base_monolith_golden_values_across_every_strategy`; `run_is_deterministic_across_repeated_calls` |
| Interrupted-replay digest equals uninterrupted digest | Domain Orchestration | `jitter-recovery` | ADR-0108 | docs/ddd/12-operations-failure.md | `interrupted_replay_matches_uninterrupted_digest_at_tick_six`; tick-9 variant |
| Zero adapter crates in dependency graph; adapters pass contract suite | Hexagonal Boundary | `jitter-ports` | ADR-0109 | docs/ddd/02-context-map.md, docs/ddd/09-upstream-integrations.md | 5 mock-adapter unit tests; `scripts/check_dependency_direction.py` |
| Mocked E2E use-case test passes with zero concrete adapters | Hexagonal Boundary | `jitter-application` | ADR-0110 | docs/ddd/10-application-workflows.md | `mocked_e2e_use_case_passes_with_zero_concrete_adapters` |
| Passes `RetryConfigSource` contract suite | Infrastructure Adapters | `jitter-adapter-aws` | ADR-0111; extends ADR-0005, ADR-0018 | docs/ddd/09-upstream-integrations.md | `passes_shared_retry_config_source_contract` + `real_retry_config_translates` |
| Passes `MemorySink` contract suite | Infrastructure Adapters | `jitter-adapter-ruvector` | ADR-0112; extends ADR-0008 | docs/ddd/08-memory-persistence.md | `passes_shared_memory_sink_contract` + real `VectorDB` round-trip |
| Passes `WitnessSink` contract suite; byte-stable digest | Infrastructure Adapters | `jitter-adapter-rvf` | ADR-0113; extends ADR-0009 | docs/ddd/07-evidence-receipts.md | `passes_shared_witness_sink_contract` + `digest_is_stable_across_repeated_seals` |
| Passes `HarnessSink` contract suite; drives recovery replay | Infrastructure Adapters | `jitter-adapter-ruflo` | ADR-0114 | docs/ddd/12-operations-failure.md | `passes_shared_harness_sink_contract`; `simulated_interruption_at_tick_six_matches_architecture_scenario` |
| Passes `FlywheelSink` contract suite | Infrastructure Adapters | `jitter-adapter-metaharness` | ADR-0115; extends ADR-0023 | docs/ddd/09-upstream-integrations.md | `passes_shared_flywheel_sink_contract` + append-only feed tests |
| Receipt independently re-verifiable without re-running the simulation | Cross-Cutting Support | `jitter-evidence` | ADR-0116 | docs/ddd/07-evidence-receipts.md | `outcome_digest_is_stable_and_reverifiable`; `serde_round_trip_is_exact` |
| Malformed config fails typed, before any adapter is constructed | Cross-Cutting Support | `jitter-config` | ADR-0117 | docs/ddd/11-security-policy.md | `missing_required_field_fails_fast`; `invalid_aws_max_attempts_fails_before_any_adapter_is_constructed` |
| One root span per tick, verified via test subscriber | Cross-Cutting Support | `jitter-observability` | ADR-0118 | docs/ddd/12-operations-failure.md | `tick_outcomes_emit_a_span_with_expected_fields` |
| Real-adapter E2E run over a service boundary | Composition Roots | `jitter-service` | ADR-0119 | docs/ddd/10-application-workflows.md | `black_box_run_endpoint_returns_a_receipt`; `evaluate_endpoint_submits_to_metaharness_and_reports_outranking` |
| CLI run exits 0; stdout matches golden fixture byte-for-byte | Composition Roots | `jitter-cli` | ADR-0120; extends ADR-0024, ADR-0025 | docs/ddd/10-application-workflows.md | `fleet_retry_storm_22_weighted_matches_committed_golden_output` against `fixtures/fleet-retry-storm-22.json` and the committed `crates/jitter-cli/tests/golden/fleet-retry-storm-22.weighted.json` |
| Weighted policy reproducibly outranks baseline, committed delta | Fitness Evaluation | `jitter-evaluation` | ADR-0121; extends ADR-0003, ADR-0017, ADR-0019, ADR-0022 | docs/ddd/01-ubiquitous-language.md (Promotion Eligibility) | `weighted_tenant_fair_outranks_unweighted_baseline_with_committed_delta` |
| All 5 adapters invoke and pass contract fn; dev-dep-only confirmed | Contract Verification | `jitter-contract-tests` | ADR-0122 | docs/ddd/02-context-map.md | Every adapter's own test suite; `scripts/check_dependency_direction.py` |
| Kill-at-tick-6 replay digest equals uninterrupted digest (combined) | Domain Orchestration + Infrastructure Adapters | `jitter-recovery`, `jitter-adapter-ruflo` | ADR-0108, ADR-0114 | docs/ddd/12-operations-failure.md | docs/architecture.md §7's exact scenario, reproduced in `jitter-recovery`'s own test suite |

Every row above is implemented and passes both gates in docs/architecture.md §8
(docs/implementation-contract.md's exact crate-counting rule). `jitter-cli`'s
`--interrupt-after`/`--replay` recovery workflow is covered by its own automated test,
`interrupted_and_replayed_run_matches_uninterrupted_receipt`
(`crates/jitter-cli/tests/recovery_cli.rs`), which runs the real binary three times
(uninterrupted, `--interrupt-after 6`, `--replay`) and asserts the uninterrupted and
replayed stdout are identical.

| Live acceptance criterion | Context | Crate | Decision | Evidence |
|---|---|---|---|---|
| Ruflo process round-trip produces bounded schema-valid provenance receipt | Infrastructure Adapters | `jitter-adapter-ruflo` | ADR-0123 | real RuOS integration test plus process failure fixtures |
| Darwin verifies frozen suite and Flywheel consumes one adapter record with verified replay | Infrastructure Adapters + Fitness Evaluation | `jitter-adapter-metaharness`, `jitter-evaluation` | ADR-0123; ADR-0023 | installed-package integration test and append-only receipt |
| Removing each process preserves the simulation digest while removing only its evidence | Domain Orchestration | `jitter-application`, both adapter crates | ADR-0123 | three-way combined/Ruflo-disabled/MetaHarness-disabled ablation |
