# Core-Memory Guidance Map (JitterMap)

Sources consulted:
- `core-memory` AGENTS blob `0b2613acb2a5be8b5f47bfe1a89f9ecfafad64d3`, read at core-memory
  repository main `f57cc3deebe89f2ec0fa2369e05ddf76a56db965`.
- `build-standard` blob `a63fd1464eaf5eecbe919aac2e7699df24eabbdc`.
- Live `task_start` memory query, namespace `dream-machine` — returned empty arrays, `unscreened=true`. Tagged **LIVE_MEMORY_EMPTY_UNSCREENED**: this is *not* retrieved guidance and contributes no decisions below; it is listed only to record that the check was attempted and came back empty/unscreened.

Both blob hashes above are unchanged from the prior guidance map
(`evidence/core-memory-guidance-map-2026-10-10.md`); only the main SHA they were read
against is newly recorded here, and only for the `core-memory` AGENTS blob, which is the
one whose source repository main is traceable. This map does not assert a `jitter-map`
repository main SHA — `jitter-map`'s own base/design SHAs are tracked in
PROJECT_STATUS.md instead, to avoid conflating an external guidance source's commit with
this project's own.

| Source/Blob | Guidance | Concrete JitterMap Decision | Planned Evidence (crate/doc/test) | Status |
|---|---|---|---|---|
| core-memory AGENTS `0b2613a` | Design-before-code | Architecture/design docs produced and reviewed before crate implementation begins (current recovery branch is design-phase) | docs/architecture.md, ADR-0027, docs/implementation-contract.md, docs/traceability.md, docs/integration-matrix.md, docs/validation.md; this map | APPLIED (design-level only) |
| build-standard `a63fd14` | Minimum 20 meaningful crates | JitterMap plans 22 crates (docs/architecture.md §5); this worktree's workspace now has all 22, each passing the exact crate-counting rule | Cargo workspace `members` list (`cargo metadata` reports 22); docs/implementation-contract.md's rule verified per-crate in docs/traceability.md | APPLIED (22/22, uncommitted in this worktree — PROJECT_STATUS.md) |
| build-standard `a63fd14` | Typed boundaries between crates | Public crate APIs use concrete types (`ValidatedScenario`, `OutcomeSummary`, `ReceiptRef`, ...), not untyped JSON/`Any` at boundaries | Crate `lib.rs` public signatures; every port in `jitter-ports` is a typed trait, not a JSON-in/JSON-out callback | APPLIED |
| build-standard `a63fd14` | One writer per shared store | `jitter-adapter-ruvector::RuVectorMemoryAdapter` is the sole writer of the RuVector-backed memory store; no other crate opens its storage path | ADR-0112; `jitter-application` only ever calls `MemorySink::record` through the one injected adapter | APPLIED |
| build-standard `a63fd14` | Real integrations, no mocks in production path | Real AWS `aws-smithy-types::RetryConfig`, real `ruvector-core::VectorDB`, real `rvf-crypto` witness chain are all live in the composition roots; `jitter-ports`' mocks are used only in `#[cfg(test)]`/dev-test code, never wired into `jitter-cli`/`jitter-service` | `jitter-adapter-aws`/`-ruvector`/`-rvf`'s own `real_*`-prefixed tests; `jitter-cli`/`jitter-service`'s composition functions name only real adapters | APPLIED |
| core-memory AGENTS `0b2613a` | Source-bound evidence | Every guidance claim in project docs must cite its source blob/commit hash | This document | APPLIED |
| core-memory AGENTS `0b2613a` | Append-only memory | Evidence/memory records are never rewritten, only appended (matches repo's existing `docs(evidence): bind ... receipt` commit pattern) | Existing evidence-bind commits (`3db7fd2`, `9684ca7`, `bed4f0b`, etc.) | APPLIED (repo convention; not yet re-verified for JitterMap crates) |
| core-memory AGENTS `0b2613a` | Evaluators cannot self-promote | No agent/evaluator may mark its own output as passing a gate; promotion requires an independent reviewer | docs/validation.md "Independent semantic review" section | PENDING — all 22 crates' Gate 1/Gate 2 passes in this worktree were authored and run by the same bounded implementation agent; no independent reviewer has signed off yet. This gap is named, not closed, by this migration. |
| core-memory AGENTS `0b2613a` | Checker plus semantic review | Automated checker AND a separate semantic/human review are both required before merge | `scripts/check_dependency_direction.py` wired into `.github/workflows/ci.yml` (Gate 1, now real); docs/validation.md "Independent semantic review" (Gate 2's human half, still PENDING) | PARTIAL — the automated half now exists and runs; the independent-human-review half does not |
| core-memory AGENTS `0b2613a` | No publication before gates | Evidence/receipt-binding commits must not land until the corresponding crate work passes its gates | This migration has not committed anything — all work described here is uncommitted in the worktree, by the task's own instruction, pending the user's review | N/A — not applicable while uncommitted; prior evidence-bind commits in history (pre-dating this migration) remain flagged as historical context only |
| Live `task_start` (dream-machine) | — | No decision sourced; query returned empty arrays, `unscreened=true` | N/A | LIVE_MEMORY_EMPTY_UNSCREENED — not retrieved guidance |
| build-standard `a63fd14` | Real integrations require executed external boundaries | Replace partial Ruflo/MetaHarness seams with bounded process runners in the existing adapter crates | ADR-0123; live integration tests; append-only receipts; ablations | PLANNED — design frozen before code |
| core-memory AGENTS `0b2613a` | Exact source-bound evidence and fail-closed authority | Pin registry version/integrity separately from source SHA; validate output and keep evaluator authority `none` | docs/integration-matrix.md; docs/validation.md; receipt schema | APPLIED (design) |
| core-memory AGENTS `0b2613a` | Coordination is not implementation | Keep ChatGPT Ruflo MCP probe separate from the Rust process round-trip | ADR-0123; Ruflo receipt and integration test | APPLIED (design) |
