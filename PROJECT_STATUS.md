# Project status

- IMPLEMENTED: opt-in `WeightedTenantFairBudget` plus the existing equal-share ablation, weighted workload validation, typed per-tenant outcomes and Jain service fairness.
- VALIDATED: local Rust 1.99.0 gates exited 0; GitHub Actions run [38103277054](https://github.com/nicholas-ruest/jitter-map/actions/runs/38103277054) passed for connector candidate `396e566e93a41501e38b8225dced04fa48f000eb`.
- REPO_PUBLISHED: implementation PR [#9](https://github.com/nicholas-ruest/jitter-map/pull/9) merged to main at `118c5f1b50aa76e6b050fcea7d717a03e35d4dad`.
- GIST_PUBLISHED: **BLOCKED_GIST_PUBLISH**. Research and build drafts are committed, but the governed GitHub connector does not expose Gist creation and no noninteractive authorized Gist publisher is available.
- Documentation inventory: 26 substantive ADRs plus index; 12 substantive DDD documents plus index.
- README: official versioned top-row marks and two project-specific animated SVGs remain present.
- Authority: none. No evaluator or memory component may promote a policy.

Frozen skewed workload (`8f5b79e0a345229171428559523a79a152b2f04a29d209e7ee68c40b40822bc7`):

| Candidate | RAF | Successes | p95 | Denied | Jain fairness |
|---|---:|---:|---:|---:|---:|
| Adaptive drop | 3.7017 | 72 | 16 | 527 | 0.1250 |
| Equal-share tenant fair | 3.7100 | 63 | 15 | 536 | 0.8932 |
| Demand-weighted tenant fair | 3.7100 | 65 | 16 | 534 | 0.9684 |

Decision: **REVISE**. Demand weighting materially improves service fairness and recovers two successes versus equal-share admission, but it still does not preserve the adaptive-drop success baseline. It is not promoted.
