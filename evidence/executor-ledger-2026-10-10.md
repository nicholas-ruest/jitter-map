# Executor ledger - 2026-10-10

## Worktree

- Repository: nicholas-ruest/jitter-map
- Base: `9684ca762bc8f831f61268b33d6b969b8de6046c`
- Branch: `recovery/2026-10-10-jittermap-weighted-fair`
- Directory: `/home/ruv/.local/share/dream-machine/worktrees/jitter-map-2026-10-10-weighted`
- Writer policy: one writer in the isolated worktree.

## Executor sequence

1. Codex CLI 0.158.0: one bounded availability probe; result `Not logged in`; no files changed.
2. Claude Code 2.1.283: authenticated through claude.ai; invoked from the worktree with `--permission-mode dontAsk --permission-prompts none --safe-mode` and allowlisted Read/Edit/Write/Grep/Glob plus Cargo, local Git inspection, rg/find, hashing, Node and npm commands. Transport timed out while the process remained live. After verification that no files had changed, the exact process was terminated. No retry.
3. Governed RuOS shell: implemented the typed demand-weighted policy, tests, evidence and documentation; then ran independent gates.

No executor received repository publication, PR, merge, Gist, deployment, force-push, secret, or policy-changing authority.

## Ruflo execution ledger

- Callable namespace: `mcp__codex_apps__ruflo_federation_federation_identity`.
- Relay: `wss://relay.ruv.io`.
- Role: coordination/provenance only; no implementation evidence attributed to Ruflo.

## Measured implementation checkpoint

- Targeted format and test command exited 0.
- Domain fairness tests: 4 passed.
- Evaluation tests: 2 passed.
- Frozen skewed digest: `8f5b79e0a345229171428559523a79a152b2f04a29d209e7ee68c40b40822bc7`.
- Weighted candidate: RAF 3.7100; successes 65; p95 16; denied 534; Jain service fairness 0.968398.
- Equal-share ablation: RAF 3.7100; successes 63; p95 15; denied 536; fairness 0.893196.
- Adaptive drop baseline: RAF 3.701667; successes 72; p95 16; denied 527; fairness 0.125.

Full workspace, dependency, evaluator and CI results are recorded after their commands complete; no passing result is inferred here.

## Full independent validation

- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`: exit 0.
- `cargo test --locked --workspace --all-targets --all-features`: exit 0; 15 tests passed, including real AWS RetryConfig and RuVector/RVF integration.
- `cargo deny check`: exit 0; advisories, bans, licenses and sources all OK.
- Weighted CLI through AWS policy source, RuVector insert/search/read-back and RVF seal: exit 0; JSON receipt recorded.
- Balanced and skewed frozen benchmarks: exit 0; raw JSON recorded.
- Darwin numeric evaluation 0.10.3: exit 0; baseline retained.
- Flywheel 0.1.12: exit 0; two generations, zero promotions, replay verification passed, authority none.

## Governed repository publication

- Local source commit: `f3cb082f9ed51b2f5ef944a26fe8969a2bcc53ce`.
- Connector candidate commit: `396e566e93a41501e38b8225dced04fa48f000eb`.
- Implementation PR: https://github.com/nicholas-ruest/jitter-map/pull/9
- GitHub Actions run 38103277054 passed the exact connector candidate.
- PR #9 merged through the governed connector to main at `118c5f1b50aa76e6b050fcea7d717a03e35d4dad`.
- Gist publishing remains blocked; publication-ready research and build files are preserved under `publication/`.
