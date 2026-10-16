# Integration matrix

This table is the honest executed-vs-not ledger for every external ingredient
JitterMap reuses, updated through ADR-0124 for the published 22-crate implementation.
"Executed" means the named integration test actually ran against the pinned external
code or process boundary. The former six-package base (`3db7fd2`) has been deleted
and superseded; docs/validation.md's checker semantics still apply. Executed status
does not imply promotion, deployment, relay membership, or publication authority.

| Ingredient | Pinned revision / version | License | Reused capability | Boundary status | Owning crate | Integration test | Ablation | Current evidence |
|---|---|---|---|---|---|---|---|---|
| AWS SDK for Rust | source `193882fe11fce9b22424ac913eeb4d03963d5700`; crate `aws-smithy-types` 1.3.4 | Apache-2.0 | `RetryConfig` standard/max-attempts semantics | **Executed**, as a standalone adapter | `jitter-adapter-aws` (docs/architecture.md §5, row 11; ADR-0111) | `real_retry_config_translates`; `passes_shared_retry_config_source_contract` | Local-jitter / AWS-standard policies run without this adapter (docs/specification.md, Degradation table) | docs/traceability.md row "AWS truth"; docs/dependency-provenance.md |
| RuVector | source `5a93328f2fceb0307c25929ed38cd7a0911fdf00`; crate `ruvector-core` 2.3.1 | MIT | `VectorDB` insert/search/get for outcome memory | **Executed**, split into its own `MemorySink`-only adapter (narrower 4-dimension embedding than the former combined package — ADR-0112) | `jitter-adapter-ruvector` (row 12; ADR-0112) | `real_vector_round_trip_and_recall`; `passes_shared_memory_sink_contract` | A run still completes and seals a `Receipt` without memory persistence (docs/specification.md, Degradation table) | docs/traceability.md row "Auditable memory" |
| RVF | crate `rvf-crypto` 0.2.0 | MIT OR Apache-2.0 | Witness chain binding and SHAKE-256 sealing | **Executed**, split into its own `WitnessSink` adapter | `jitter-adapter-rvf` (row 13; ADR-0113) | `real_witness_chain_seals_and_verifies`; `passes_shared_witness_sink_contract` | An unsealed `OutcomeSummary` is still inspectable without a witness root (docs/specification.md, Degradation table) | docs/traceability.md row "Auditable memory" |
| MetaHarness (Darwin 0.10.3 / Flywheel 0.1.12) | inspected source `9ce8b8dd89045c3b9a1f809ae58f3589029db4a4`; registry versions 0.10.3 / 0.1.12 | MIT | Bounded Darwin numeric evolution and Flywheel replay evaluation | **Executed.** The Rust adapter launched the pinned installed Node artifacts against the real `jitter-candidate` evaluator; Flywheel consumed the JSONL feed and verified replay with zero promotions. npm omits `gitHead`, so registry-version and source-commit provenance remain separate facts. | `jitter-adapter-metaharness` (row 15; ADR-0115, ADR-0123) | live ignored process round-trip; failure fixtures; `passes_shared_flywheel_sink_contract` | frozen Rust scoring without the external consumer | live integration receipt and CI-bound PR #13 |
| Ruflo coordination | `@claude-flow/cli` 3.56.10; MIT; npm integrity `sha512-jcl7i1gwnb1vQRlGwmXU/FkBulxf3nKhYPC6cZiDPn0fptw6OJgSSsgAQf229axUcqsmrA3pd3V+ma0vO7Xv+A==` | MIT | Registry discovery, read-only signed public federation sync, and local interruption signal | **Executed, bounded read-only.** Fixed argv, 30-second timeout, 64 KiB output ceilings, fail-closed schema validation, opaque untrusted bodies, aggregate receipts, and `authority=none`. No relay join, publication, promotion, deployment, or remote interruption authority. | `jitter-adapter-ruflo` (row 14; ADR-0114, ADR-0123, ADR-0124) | live registry plus seven-day sync; timeout/non-zero/schema/output-cap fixtures; contract and recovery tests | direct CLI recovery without Ruflo; registry-only path without sync | PR #15, CI run 38838919923, research/build Gist addenda |

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
- Ruflo and MetaHarness are now **Executed** through bounded process boundaries.
  Ruflo remains deliberately read-only with authority `none`; MetaHarness remains
  advisory-only with zero promotions. These statuses prove execution, not production
  readiness, deployment authority, or npm-to-source cryptographic identity.

## ADR-0123 implementation target

| Ingredient | Exact runtime pin | Executed boundary | Required test | Ablation |
|---|---|---|---|---|
| Ruflo | `@claude-flow/cli` 3.56.10; MIT; npm integrity `sha512-jcl7i1gwnb1vQRlGwmXU/FkBulxf3nKhYPC6cZiDPn0fptw6OJgSSsgAQf229axUcqsmrA3pd3V+ma0vO7Xv+A==` | Rust `Command` to a fixed CLI provenance operation; ChatGPT MCP identity is independent corroboration | installed CLI round-trip plus missing/timeout/non-zero/schema/output-cap fixtures | domain replay without process provenance |
| Darwin | 0.10.3; inspected source `9ce8b8dd89045c3b9a1f809ae58f3589029db4a4`; MIT | Rust launches Node and pinned CLI `evolve-numeric` with frozen genome, seed, bounds, and real `jitter-candidate` evaluator | real numeric generation plus all process failure fixtures | local Rust scoring only |
| Flywheel | 0.1.12; same inspected source pin; MIT | repository ESM bridge imports installed module, consumes exact JSONL, runs one bounded generation, verifies replay bundle | real feed-to-generation-to-verify round-trip | append-only feed without external consumer |

The npm packages do not expose `gitHead`; receipts must report the registry versions
and the separately inspected source pin without claiming they are cryptographically
identical.
