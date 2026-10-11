# Dream Machine build announcement - 2026-10-10 - JitterMap weighted fairness

Recovery publication date: 2026-10-10. This file is publication-ready but is not a published Gist.

## Capability

JitterMap now evaluates demand-weighted tenant admission during correlated retry storms while preserving the existing equal-share tenant policy as an explicit ablation. The complete path accepts a frozen fleet scenario, sources enterprise retry behavior from the AWS SDK for Rust, executes policy admission, persists searchable outcome evidence in RuVector, and seals the receipt with RVF.

## Exact ingredients

- RuVector / RVF: ruvnet/ruvector commit `5a93328f2fceb0307c25929ed38cd7a0911fdf00`; MIT and MIT OR Apache-2.0 components.
- AWS SDK for Rust: awslabs/aws-sdk-rust commit `193882fe11fce9b22424ac913eeb4d03963d5700`; Apache-2.0.
- MetaHarness Darwin and Flywheel: ruvnet/metaharness commit `9ce8b8dd89045c3b9a1f809ae58f3589029db4a4`; npm artifacts Darwin 0.10.3 and Flywheel 0.1.12.
- Frontier mechanism: request-normalized Jain service fairness applied to demand-weighted round-robin retry admission, evaluated under frozen correlated-failure scenarios.

## Architecture

- [Animated overview](https://github.com/nicholas-ruest/jitter-map/blob/118c5f1b50aa76e6b050fcea7d717a03e35d4dad/docs/assets/overview-hero.svg)
- [Animated execution flow](https://github.com/nicholas-ruest/jitter-map/blob/118c5f1b50aa76e6b050fcea7d717a03e35d4dad/docs/assets/execution-flow.svg)
- [Architecture document](https://github.com/nicholas-ruest/jitter-map/blob/118c5f1b50aa76e6b050fcea7d717a03e35d4dad/docs/architecture.md)

## Reproducible validation

Validated implementation candidate: `396e566e93a41501e38b8225dced04fa48f000eb`.
Merged main: `118c5f1b50aa76e6b050fcea7d717a03e35d4dad`.
GitHub Actions run: https://github.com/nicholas-ruest/jitter-map/actions/runs/38103277054

Commands with exit code 0:

- `cargo fmt --all -- --check`
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`
- `cargo test --locked --workspace --all-targets --all-features` (15 tests)
- `cargo deny check`
- weighted CLI vertical slice through AWS policy sourcing, RuVector insert/search/readback, and RVF seal
- frozen balanced and skewed benchmarks
- Darwin 0.10.3 numeric evaluation
- Flywheel 0.1.12 two-generation replay verification

Frozen skewed workload digest: `8f5b79e0a345229171428559523a79a152b2f04a29d209e7ee68c40b40822bc7`.

| Candidate | RAF | Successes | p95 | Denied | Jain fairness |
|---|---:|---:|---:|---:|---:|
| Adaptive drop | 3.7017 | 72 | 16 | 527 | 0.1250 |
| Equal-share tenant fair | 3.7100 | 63 | 15 | 536 | 0.8932 |
| Demand-weighted tenant fair | 3.7100 | 65 | 16 | 534 | 0.9684 |

## Result and limitations

Demand weighting improved fairness from 0.8932 to 0.9684 and recovered two successes versus equal-share admission. It did not preserve adaptive drop's 72-success baseline, so Darwin retained the baseline and Flywheel recorded zero promotions. The policy has no cross-tick deficit carry-over. Receipt integrity is hash-linked and RVF-sealed but is not externally anchored. The evaluator role has no promotion authority.

## Links

- Repository: https://github.com/nicholas-ruest/jitter-map
- Exact merged commit: https://github.com/nicholas-ruest/jitter-map/commit/118c5f1b50aa76e6b050fcea7d717a03e35d4dad
- Implementation PR: https://github.com/nicholas-ruest/jitter-map/pull/9
- Research report draft: [dream-machine-research-2026-10-10-jitter-map-weighted-fairness.md](dream-machine-research-2026-10-10-jitter-map-weighted-fairness.md)

Research and build Gist URLs remain unavailable: **BLOCKED_GIST_PUBLISH**.
