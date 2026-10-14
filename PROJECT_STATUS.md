# Project status

## PARTIAL_FAILURE / INTEGRATION_BLOCKED

JitterMap is now a substantive, published 22-crate Rust system. The independently validated RuOS tree was reproduced exactly on GitHub, passed CI, and merged through PR #11. The dated research and build Gists were published and read back as public artifacts owned by `nicholas-ruest`.

It is not marked `BUILT` because two declared optional adapter boundaries remain incomplete: the Ruflo adapter has no demonstrated live remote transport, and the MetaHarness adapter has no demonstrated external Darwin/Flywheel process round-trip. The mandatory executed composition is nevertheless present through AWS Smithy, RuVector and RVF.

## Immutable delivery

- Merged main: `1c42e7adbaa99988598592e91b0c2f2489087f42`
- Git tree: `5dcfef7348cf03631cc59f1df070712fe1c24923`
- Pull request: https://github.com/nicholas-ruest/jitter-map/pull/11
- Exact candidate: `bf6a9aa8b56af39bbfa2d14d6df3aa4cc7fb578d`
- CI run: https://github.com/nicholas-ruest/jitter-map/actions/runs/38497481659
- Research Gist: https://gist.github.com/nicholas-ruest/9806bb7dcbcb5c86b43005ff31b5a07f
- Build Gist: https://gist.github.com/nicholas-ruest/92ae66b6aa97644d35771fa40e711dc4

## Observed inventory

- 22 implemented Rust workspace packages
- 4,595 Rust source lines
- 49 ADRs plus indexes
- 12 DDD documents plus indexes
- 2 project-specific animated SVGs
- 107 locked workspace tests passed locally
- GitHub CI passed dependency policy, formatting, strict Clippy, locked tests, dependency direction and two benchmarks

## Executed composition

- AWS Smithy retry configuration translation
- RuVector 2.3.1 VectorDB insert/search/get
- RVF Crypto 0.2.0 witness seal/verify and tamper rejection
- Adaptive deferral and demand-weighted fairness ablations
- Darwin 0.10.3 suite/hash verification
- Flywheel 0.1.12 bounded generation and replay verification
- Ruflo Federation provenance probe

## Guidance compliance

- REPOSITORY_GUIDANCE_APPLIED at core-memory `f57cc3deebe89f2ec0fa2369e05ddf76a56db965`
- LIVE_MEMORY_EMPTY_UNSCREENED: the live task-start and risk calls returned no records with `unscreened=true`; no live preference retrieval was claimed
- APPLIED: design-first documentation, typed boundaries, one-writer worktree, append-only evidence, exact commit binding, advisory evaluators, PR/CI merge discipline
- DEVIATED: live Ruflo and MetaHarness adapter round-trips remain incomplete and are explicitly reported

## Decision

**REVISE.** Continue this project by implementing and executing the two live adapter transports. Do not create another primitive repository.
