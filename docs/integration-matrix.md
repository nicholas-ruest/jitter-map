# Integration matrix

This table is the honest executed-vs-not ledger for every external ingredient
JitterMap reuses, now updated for the 22-crate implementation (ADR-0101–ADR-0122).
"Executed" means the integration test named has actually run against real code in
this worktree; "Partial" means a real port implementation exists and passes its
contract suite, but no live external transport/process exists to round-trip with (the
Ruflo and MetaHarness rows — see each row's own note for exactly what is and is not
proven). The former six-package base (`3db7fd2`) this table once described has been
deleted and superseded; docs/validation.md's checker semantics still apply.

| Ingredient | Pinned revision / version | License | Reused capability | Boundary status | Owning crate | Integration test | Ablation | Current evidence |
|---|---|---|---|---|---|---|---|---|
| AWS SDK for Rust | source `193882fe11fce9b22424ac913eeb4d03963d5700`; crate `aws-smithy-types` 1.3.4 | Apache-2.0 | `RetryConfig` standard/max-attempts semantics | **Executed**, as a standalone adapter | `jitter-adapter-aws` (docs/architecture.md §5, row 11; ADR-0111) | `real_retry_config_translates`; `passes_shared_retry_config_source_contract` | Local-jitter / AWS-standard policies run without this adapter (docs/specification.md, Degradation table) | docs/traceability.md row "AWS truth"; docs/dependency-provenance.md |
| RuVector | source `5a93328f2fceb0307c25929ed38cd7a0911fdf00`; crate `ruvector-core` 2.3.1 | MIT | `VectorDB` insert/search/get for outcome memory | **Executed**, split into its own `MemorySink`-only adapter (narrower 4-dimension embedding than the former combined package — ADR-0112) | `jitter-adapter-ruvector` (row 12; ADR-0112) | `real_vector_round_trip_and_recall`; `passes_shared_memory_sink_contract` | A run still completes and seals a `Receipt` without memory persistence (docs/specification.md, Degradation table) | docs/traceability.md row "Auditable memory" |
| RVF | crate `rvf-crypto` 0.2.0 | MIT OR Apache-2.0 | Witness chain binding and SHAKE-256 sealing | **Executed**, split into its own `WitnessSink` adapter | `jitter-adapter-rvf` (row 13; ADR-0113) | `real_witness_chain_seals_and_verifies`; `passes_shared_witness_sink_contract` | An unsealed `OutcomeSummary` is still inspectable without a witness root (docs/specification.md, Degradation table) | docs/traceability.md row "Auditable memory" |
| MetaHarness (Darwin 0.10.3 / Flywheel 0.1.12) | inspected source `9ce8b8dd89045c3b9a1f809ae58f3589029db4a4` | MIT | Bounded Darwin proposal scoring and Flywheel replay evaluation | **Partial.** `FlywheelSink` is implemented and passes its contract suite, appending real evidence to an append-only JSON-Lines feed (ADR-0115); there is still no live round-trip with an actual Darwin/Flywheel process reading that feed, because none is reachable from this environment. **Provenance gap, flagged honestly, unchanged by this migration**: the inspected source commit above was read directly from the MetaHarness source tree; the corresponding npm package's `gitHead` field is **absent**, so there is no package-metadata cross-check tying the published npm artifact to this exact commit. The pin is source-commit evidence only, not npm-attested provenance. | `jitter-adapter-metaharness` (row 15; ADR-0115) | `passes_shared_flywheel_sink_contract`; `jitter-service`'s `/evaluate` endpoint submits a real scored candidate | Darwin/Flywheel evaluation is advisory-only and never required for a run to complete (docs/specification.md, Degradation table; ADR-0023 safety envelope) | docs/traceability.md row "Frozen comparison" |
| Ruflo coordination | **No pin exists**, unchanged by this migration. No source revision, package version, or license has been inspected for Ruflo as a dependency; the only Ruflo surface used anywhere in this repository is an MCP coordination/provenance channel, not a library dependency | N/A — not vendored | `HarnessSink` driving `jitter-recovery`'s interruption trigger | **Partial.** `HarnessSink` is implemented with a real, thread-safe atomic interruption-signal primitive (`RufloSignalAdapter`, ADR-0114) and passes its contract suite; there is still no live Ruflo network transport behind it, because none exists to integrate against. This must not be read as "Ruflo is integrated" — only "the port Ruflo would eventually be wired through is implemented and proven with a simulated signal." | `jitter-adapter-ruflo` (row 14; ADR-0114) | `passes_shared_harness_sink_contract`; `simulated_interruption_at_tick_six_matches_architecture_scenario`; `jitter-application`'s `execute_with_recovery_routes_through_checkpoint_replay_on_interruption` drives a real `jitter-recovery` replay from this adapter's signal | Recovery can still be triggered directly (`jitter-cli --interrupt-after`) without any orchestration layer | `jitter-recovery`'s own test suite; see ADR-0114 |

## Reading this table honestly

- "Executed" on AWS, RuVector, and RVF rows now means the standalone adapter crates
  (`jitter-adapter-aws`, `jitter-adapter-ruvector`, `jitter-adapter-rvf`) exist as real
  workspace members — `cargo metadata` reports 22 packages, not the former base six
  (ADR-0027's consequence is now realized, not merely planned).
- MetaHarness's provenance gap (absent npm `gitHead`) is not a blocker to using the
  inspected source commit as the pin of record, but it must be carried forward in every
  document that cites this pin — docs/dependency-provenance.md and docs/validation.md
  both reference this same caveat rather than silently dropping it. It is unaffected by
  this migration either way: `jitter-adapter-metaharness` builds a real evidence feed,
  not a new relationship with the MetaHarness project's own publishing practices.
- The Ruflo and MetaHarness rows are marked **Partial**, not **Executed**: a real,
  tested port implementation exists for both, but neither has a live external
  process on the other end to prove a true round-trip against. Do not read "Partial"
  as "integrated," "wired," or "adapter-complete" in the sense of a working live
  connection — only as "the port contract is honestly implemented and proven with a
  realistic simulated signal."
