# Live integration receipt — 2026-10-14

## Scope

This receipt closes the two external-process gaps identified in `PROJECT_STATUS.md` at base commit `0ab974db68ea097d173a6de282315dc16304360b`. It is source-bound to the candidate commit recorded after validation; until then the candidate state is the indexed worktree diff from that base.

## Environment

- Working directory: `/home/ruv/.local/share/dream-machine/worktrees/jitter-map-2026-10-11-design`
- Node: `v24.21.0`
- rustc: `1.99.0 (b940084d7 2026-09-28)`
- Cargo: `1.99.0 (5f94df478 2026-08-27)`
- Ruflo CLI: `@claude-flow/cli@3.56.10`, MIT, npm integrity `sha512-jcl7i1gwnb1vQRlGwmXU/FkBulxf3nKhYPC6cZiDPn0fptw6OJgSSsgAQf229axUcqsmrA3pd3V+ma0vO7Xv+A==`
- Ruflo CLI entry SHA-256: `8a1020f603b35cbcabd37279bcaaf8934ff690337c0a9e8e20977bccbfddbebd`
- MetaHarness Flywheel: `@metaharness/flywheel@0.1.12`
- Flywheel ESM entry SHA-256: `965ddba1b676b57069ceb076f9d26a7183cee0bada66462d3825c6876359c982`
- Repository bridge SHA-256: `ee8493de814bf2a9f64c5dca05e16c174a6ef8eda14de5e293c7ba65cee3d1c3`

## Ruflo live registry boundary

The Rust integration test constructed `RufloProcessTransport` with canonical executable and repository paths. The transport used fixed argv (`federation registry --format json`) without a shell, applied a 30-second timeout and 64 KiB output cap, validated TLS endpoints and the typed registry schema, omitted the gateway public key from its receipt, and recorded `authority=none`.

```text
command: JITTER_RUFLO_BIN=.../node_modules/.bin/claude-flow JITTER_RUFLO_WORKDIR=$PWD cargo test --locked -p jitter-adapter-ruflo --lib tests::live_ruflo_registry_when_configured -- --ignored --exact --nocapture
exit: 0
tests: 1 passed; 0 failed
observed schema: relay=wss://relay.ruv.io, httpBase=https://relay.ruv.io, swarmTag=ruflo-swarm, registration.enabled=true
```

The endpoint values above are public service metadata. No account, session, credential, gateway key, or private payload is recorded.

## MetaHarness Darwin/Flywheel live process boundary

The Rust integration test constructed `MetaharnessProcessRunner` with canonical Node, Darwin CLI, Darwin genome, candidate evaluator, repository bridge, repository, and installed Flywheel entry paths. The runner used fixed argv without a shell and applied a 60-second timeout plus a streaming 64 KiB cap to each child. Darwin 0.10.3 executed one deterministic numeric generation with two children and the real `target/debug/jitter-candidate` evaluator. The repository bridge then imported and called `runFlywheelGenerations`, `makeSigner`, and `verifyReplayBundle`, evaluated the same candidate binary, and wrote `evidence/flywheel-replay.json`.

```text
command: JITTER_NODE_BIN=.../node JITTER_DARWIN_CLI=.../@metaharness/darwin/dist/cli.js JITTER_DARWIN_GENOME=$PWD/evaluation/darwin-genome.json JITTER_CANDIDATE_BIN=$PWD/target/debug/jitter-candidate JITTER_METAHARNESS_SCRIPT=$PWD/evaluation/flywheel-evaluate.mjs JITTER_REPO=$PWD METAHARNESS_FLYWHEEL_ENTRY=.../@metaharness/flywheel/dist/index.js cargo test --locked -p jitter-adapter-metaharness --lib tests::live_metaharness_round_trip_when_configured -- --ignored --exact --nocapture
exit: 0
tests: 1 passed; 0 failed
validated output: Darwin package=@metaharness/darwin@0.10.3 with a non-empty winner and output digest; Flywheel package=@metaharness/flywheel@0.1.12, promotions=0, replayVerdict.pass=true, authority=none
```

## Failure-path and contract evidence

```text
command: cargo test --locked -p jitter-adapter-ruflo -p jitter-adapter-metaharness
exit: 0
result: 13 passed; 0 failed; live and child-process fixtures intentionally ignored in the default suite

command: cargo clippy --locked -p jitter-adapter-ruflo -p jitter-adapter-metaharness --all-targets --all-features -- -D warnings
exit: 0
```

The default suite exercises unavailable executables, timeouts, non-zero exits, malformed schemas, oversized output, receipt I/O failure, and both shared port contracts. The opt-in live tests are isolated because they require installed external artifacts and network availability; they were explicitly run in this receipt.

## Authority and limitations

- Both integrations are read/evaluate-only and have `authority=none`.
- The Ruflo boundary proves public registry discovery, not private-channel publication or interruption delivery from a remote peer.
- A later repeat after deleting Ruflo's generated policy-anchor files failed closed with `policy-anchor-log-missing`. Ruflo policy initialization requires an interactive local terminal, so the initial successful Rust round-trip is retained as evidence while fresh reruns require that project-local policy prerequisite to be re-established. No interactive guard was bypassed.
- The Flywheel boundary proves real external generation/replay execution with a frozen local evaluator and zero promotions. It does not grant promotion, deployment, or publication authority.
- npm did not expose `gitHead` for the installed Flywheel artifact; registry version and entry hash are therefore recorded separately from the previously inspected source revision.
