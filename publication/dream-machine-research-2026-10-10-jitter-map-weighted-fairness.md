# Dream Machine research - 2026-10-10 - proportional retry fairness

Recovery publication date: pending. Original experiment date: 2026-10-10.

## Question

Can fleet retry admission preserve per-tenant service fairness under highly skewed demand without giving up the bounded retry amplification of a shared adaptive budget?

## Primary sources and inspected OSS

- Retry Amplification in Distributed Systems, arXiv:2608.25403 (2026-08-26): https://arxiv.org/abs/2608.25403
- AWS SDK for Rust, Apache-2.0, pinned commit `193882fe11fce9b22424ac913eeb4d03963d5700`: https://github.com/awslabs/aws-sdk-rust
- RuVector, pinned source `5a93328f2fceb0307c25929ed38cd7a0911fdf00`: https://github.com/ruvnet/RuVector
- MetaHarness, pinned source `9ce8b8dd89045c3b9a1f809ae58f3589029db4a4`: https://github.com/ruvnet/metaharness

## Candidate comparison

| Candidate | Result |
|---|---|
| Keep equal-share fairness only | rejected as final answer; it over-serves low-demand tenants |
| Static per-tenant slices | rejected; unused capacity and policy coupling |
| Learned optimizer | rejected for this increment; higher authority and reproducibility risk |
| Demand-weighted round robin plus executable equal-share ablation | selected; bounded and deterministic |

## Integration matrix

| Ingredient | Reused capability | Boundary | Executed evidence | Ablation |
|---|---|---|---|---|
| AWS SDK for Rust | real `RetryConfig` | Cargo / AWS adapter | `real_retry_config_translates` | local/adaptive policies |
| RuVector | outcome insert/search/read-back | Cargo / Ruvnet adapter | real vector round trip | no-memory path |
| RVF crypto 0.2.0 | receipt witness | Cargo / Ruvnet adapter | seal and tamper test | unsealed report |
| MetaHarness Darwin/Flywheel | bounded scoring and signed replay | subprocess / evaluation | baseline retained; replay verified | frozen Rust benchmark |
| RAF research mechanism | shared budget plus measurable amplification | domain/evaluation | frozen combined baselines | AWS/local/drop/deferral/fair |

## Hypothesis and falsifiable workload

For 600 requests assigned by weights `[8,1,1,1,1,1,1,1]`, a tick-local weighted quantum should improve Jain fairness of per-tenant success ratios over equal-share round robin without reducing aggregate successes.

Result on scenario SHA-256 `8f5b79e0a345229171428559523a79a152b2f04a29d209e7ee68c40b40822bc7`: equal-share fairness 0.893196 with 63 successes; weighted fairness 0.968398 with 65 successes; both RAF 3.7100. The bounded hypothesis passed. Adaptive drop still produced 72 successes, so the promotion gate failed.

## Handoff

Keep the weighted policy opt-in and authority-none. Next work should test persistent deficit carry-over and broader demand distributions before any production claim. Repository: https://github.com/nicholas-ruest/jitter-map
