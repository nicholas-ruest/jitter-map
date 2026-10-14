# Publication receipt - 2026-10-13

## Repository

- Repository: https://github.com/nicholas-ruest/jitter-map
- PR #11: https://github.com/nicholas-ruest/jitter-map/pull/11
- PR state read back: merged
- Candidate SHA: `bf6a9aa8b56af39bbfa2d14d6df3aa4cc7fb578d`
- CI run: https://github.com/nicholas-ruest/jitter-map/actions/runs/38497481659
- CI conclusion: success
- Merged main SHA: `1c42e7adbaa99988598592e91b0c2f2489087f42`
- Merged Git tree: `5dcfef7348cf03631cc59f1df070712fe1c24923`
- Local validated Git tree: `5dcfef7348cf03631cc59f1df070712fe1c24923`
- Tree equality: exact

## Structural and semantic evidence

- Pinned checker source: `ruvnet-dream-machine@80f006f6960bc4e491e97f51e890ff708e81aa88`
- Checker result: `STRUCTURAL_PASS_REQUIRES_EVIDENCE_REVIEW`
- Counts: 49 ADRs, 12 DDDs, 22 crates, 2 animated SVGs
- Semantic review: every library has a reverse workspace caller; `jitter-cli` and `jitter-service` are executable roots; all crates contain tests or shared contract helpers
- Local gates: format, strict Clippy, 107 locked tests, cargo-deny, dependency direction, CLI workload and both benchmarks passed
- GitHub CI gates: cargo-deny, format, strict Clippy, locked workspace tests, dependency direction and both benchmarks passed

## Public research publication

- URL: https://gist.github.com/nicholas-ruest/9806bb7dcbcb5c86b43005ff31b5a07f
- Owner readback: `nicholas-ruest`
- Visibility readback: public
- File: `dream-machine-research-2026-10-13-jitter-map-system-recovery.md`
- Content checks: candidate comparison, selected hypothesis, pinned ingredients, falsifiable workload, executable integration matrix and merged commit all present

## Public build publication

- URL: https://gist.github.com/nicholas-ruest/92ae66b6aa97644d35771fa40e711dc4
- Owner readback: `nicholas-ruest`
- Visibility readback: public
- File: `dream-machine-build-2026-10-13-jitter-map-22-crate-recovery.md`
- Content checks: repository/PR/main SHA, exact CI run, research link, ingredients, architecture, measured results and limitations all present

## README readback

GitHub rendered the merged README in the authenticated light-theme session. The aligned Rust, AWS SDK for Rust, RuVector + RVF and MetaHarness marks loaded, as did the overview hero. The accessibility tree confirmed the execution-flow graphic, quickstart, workspace map, composition table, benchmark tables and limitation section.

## Remaining integration gaps

- Ruflo typed adapter exists and its local primitive is tested, but live remote transport is not demonstrated.
- MetaHarness adapter writes an append-only feed, while Darwin/Flywheel were executed separately; an external process round-trip through the adapter is not demonstrated.
- No evaluator or memory component has promotion authority.
