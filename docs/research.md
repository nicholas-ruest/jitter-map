# Research — October 4 recovery

## Frontier stream

*Retry Amplification in Distributed Systems* (arXiv:2608.25403, 2026-08-26) defines RAF and evaluates Adaptive Retry Budgeting across 200 open-source microservices. It motivates the mechanism; JitterMap does not claim to reproduce the paper.

ADR-0026 (2026-10-09) adds a second admission strategy inspired by that mechanism's distinction between a budget that is permanently exhausted versus one that is only temporarily below cost. `AdaptiveDeferral` reschedules a retry to the tick at which projected refill would clear it instead of dropping it immediately, and falls back to the drop behavior when refill is impossible or would exceed the horizon. On the frozen fixture this is not a universal improvement over the existing drop ablation — see the benchmark table in README.md — which is reported as measured rather than adjusted toward a preferred outcome.

## Enterprise OSS stream

AWS SDK for Rust commit `193882fe11fce9b22424ac913eeb4d03963d5700` (2026-10-02, Apache-2.0) supplies the real RetryConfig interface. AWS's current Bedrock traffic-shaper sample provides central pacing prior art. NVIDIA NeMo Agent Toolkit issue #2212 (2026-09-04) records adapters ignoring an explicit no-retry setting, motivating executable configuration-truth tests.

## Ruvnet

RuVector was inspected at `5a93328f2fceb0307c25929ed38cd7a0911fdf00`; ruvector-core stores prior outcomes and RVF crypto 0.2.0 seals receipts. MetaHarness was inspected at `9ce8b8dd89045c3b9a1f809ae58f3589029db4a4`; Darwin 0.10.3 and Flywheel 0.1.12 are advisory evaluators.

| Candidate | Novelty | Enterprise fit | Ruvnet fit | Executable | Risk | Total |
|---|---:|---:|---:|---:|---:|---:|
| Fleet retry admission | 5 | 5 | 5 | 5 | 3 | 23 |
| Model traffic shaper | 3 | 5 | 3 | 4 | 3 | 18 |
| Config linter | 2 | 4 | 2 | 5 | 2 | 15 |
| Learned retry optimizer | 4 | 3 | 4 | 2 | 5 | 12 |
