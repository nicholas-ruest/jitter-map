# Dream Machine research — 2026-10-04 — JitterMap

## Selected hypothesis

Independent client retry policies can be individually reasonable while producing fleet-level retry amplification under correlated failure. A deterministic admission evaluator that combines an enterprise SDK retry model with shared-budget policy, auditable Ruvnet memory and immutable witnesses can expose that tradeoff before production rollout.

## Candidate comparison

| Candidate | Novelty | Source-grounded composition | Bounded evaluation | Decision |
|---|---:|---:|---:|---|
| Fleet retry admission | High | AWS SDK + RuVector + RVF + MetaHarness | Strong | Selected |
| Client-local jitter visualizer | Low | Minimal | Strong | Rejected as too small |
| Bedrock traffic-shaper clone | Low | Mostly one upstream | Moderate | Rejected as duplicative |
| Generic resilience dashboard | Low | Decorative risk | Weak | Rejected |

## Primary research

- Retry Amplification in Distributed Systems, arXiv:2608.25403, published 2026-08-26. The paper formalizes retry amplification and reports that naïve retries can reduce success under correlated failure.
- AWS traffic-shaper sample and retry guidance: centralized admission and pacing are complementary to client-local jitter.
- NVIDIA NeMo Agent Toolkit issue #2212, opened 2026-09-04: a concrete report that configured retry disabling could be ignored, producing five attempts.
- NVIDIA resiliency-ext material updated in August 2026: retry observability and fault-handling mechanisms.

## Inspected OSS and provenance

| Component | Revision/version | License | Inspected capability |
|---|---|---|---|
| AWS SDK for Rust | `193882fe11fce9b22424ac913eeb4d03963d5700` | Apache-2.0 | `aws_smithy_types::retry::RetryConfig` |
| RuVector | source `5a93328f2fceb0307c25929ed38cd7a0911fdf00`, crate 2.3.1 | MIT | `VectorDB` insert/search/get |
| RVF crypto | 0.2.0 | MIT OR Apache-2.0 | witness creation and verification |
| MetaHarness | `9ce8b8dd89045c3b9a1f809ae58f3589029db4a4` | MIT | Darwin numeric evolution and Flywheel replay |

## Ruvnet composition

RuVector stores append-only outcome evidence and supports nearest-prior readback. RVF seals the receipt chain. Darwin explores numeric policy candidates; Flywheel records signed lineage and replays the frozen promotion rule. Every output carries `authority: none`; no evaluator can publish or activate a policy.

## Frozen evaluation

The frozen correlated-throttling fixture uses 400 requests, eight tenants, seed 20261004 and one shared scenario digest. Results:

| Strategy | RAF | Successes | p95 | Attempts | Denied |
|---|---:|---:|---:|---:|---:|
| Local full jitter | 5.4375 | 364 | 24 | 2,175 | 0 |
| AWS standard | 3.0000 | 0 | 0 | 1,200 | 0 |
| Adaptive budget | 5.3050 | 332 | 24 | 2,122 | 38 |

The adaptive policy reduced retry amplification by 2.4% but lost 8.8% of successful outcomes. It does not clear a production-promotion gate. The project is retained as an evaluation system; the current policy decision is REVISE.

## Rejected alternatives

Keeping JitterMap as a collision calculator was rejected because it did not justify Ruvnet or enterprise integration. Reimplementing AWS retry middleware was rejected because the SDK already owns client-local behavior. Automatic policy deployment was rejected because it would violate the safety envelope and turn evaluator output into authority.

## Repository

https://github.com/nicholas-ruest/jitter-map

This report contains no secrets, credentials, private source, personal data or confidential material.

## Addendum — 2026-10-09

ADR-0026 evaluates a second adaptive candidate, `AdaptiveDeferral`, inspired by the same cited mechanism's distinction between temporary and permanent budget exhaustion: instead of dropping an under-budget retry, it is deterministically rescheduled to the tick at which projected refill would cover its cost, bounded by the scenario horizon and `MAX_EVENTS`, and fails closed (drops) when refill is structurally impossible or would exceed the horizon. On the same frozen fixture and digest:

| Strategy | RAF | Successes | p95 | Attempts | Denied | Deferred |
|---|---:|---:|---:|---:|---:|---:|
| Local full jitter | 5.4375 | 364 | 24 | 2,175 | 0 | 0 |
| AWS standard | 3.0000 | 0 | 0 | 1,200 | 0 | 0 |
| Adaptive shared budget (drop) | 5.3050 | 332 | 24 | 2,122 | 38 | 0 |
| Adaptive deferral | 5.4425 | 329 | 24 | 2,177 | 0 | 55 |

Deferral converts every budget-exhaustion event into a reschedule on this fixture (0 denied) but spends more of the fixed `max_attempts` ceiling waiting on contested budget, which raises attempts and lowers successes relative to the drop ablation. This is a genuine, executed, mixed-to-negative result, not a success story adjusted after the fact. Neither adaptive candidate clears the promotion gate; the decision remains REVISE for both.
