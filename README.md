<p align="center">
  <a href="https://www.rust-lang.org/"><img src="docs/assets/marks/rust-wordmark-v1.svg" width="150" height="46" alt="Rust language"></a>&nbsp;&nbsp;
  <a href="https://github.com/awslabs/aws-sdk-rust"><img src="docs/assets/marks/aws-sdk-rust-wordmark-v1.svg" width="220" height="46" alt="AWS SDK for Rust"></a>&nbsp;&nbsp;
  <a href="https://github.com/ruvnet/RuVector"><img src="docs/assets/marks/ruvector-rvf-wordmark-v1.svg" width="200" height="46" alt="RuVector and RVF"></a>&nbsp;&nbsp;
  <a href="https://github.com/ruvnet/metaharness"><img src="docs/assets/marks/metaharness-wordmark-v1.svg" width="190" height="46" alt="MetaHarness"></a>
</p>

# JitterMap

Fleet-level retry admission with replayable evidence. JitterMap answers a system question that client-local backoff cannot: when many independently reasonable clients encounter one correlated failure, which retries should the fleet admit, defer, or deny?

![JitterMap fleet retry admission overview](docs/assets/overview-hero.svg)

## Capabilities

- deterministic discrete-event simulation with seeded full jitter;
- local, real AWS RetryConfig, adaptive shared-budget (drop), and adaptive deferral strategies;
- Retry Amplification Factor, success, denial, deferral, and p95 evidence;
- real RuVector outcome insert/search/read-back;
- real RVF witness-chain sealing;
- bounded MetaHarness Darwin/Flywheel evaluation with authority none.

## Quickstart

    cargo run --locked -- --scenario examples/correlated-outage.json --policy adaptive --memory .jitter-map-memory
    cargo run --locked -- --scenario examples/correlated-outage.json --policy adaptive-deferral --memory .jitter-map-memory
    cargo run --locked -p jitter-evaluation --bin jitter-benchmark

`--policy` accepts `local`, `aws`, `adaptive` (shared budget, drop on exhaustion), or `adaptive-deferral` (shared budget, defer on temporary exhaustion, fail closed otherwise; see [ADR-0026](docs/adrs/0026-adaptive-deferral-admission.md)). The JSON receipt includes the exact scenario digest, report, nearest prior outcome IDs, witness root, and authority none. It is evidence, not a production change.

MetaHarness evaluation uses the same compiled domain engine through `jitter-candidate`. Darwin supplies a numeric genome over stdin; Flywheel evaluates concrete policies, seals lineage, replays the frozen gate, and records zero autonomous promotions.

## Execution and data flow

![JitterMap execution and data flow](docs/assets/execution-flow.svg)

## Workspace

| Crate | Responsibility |
|---|---|
| jitter-domain | typed scenario/policy model and deterministic engine |
| jitter-application | workflow and ports |
| jitter-adapter-aws | real AWS Smithy RetryConfig translation |
| jitter-adapter-ruvnet | RuVector memory and RVF sealing |
| jitter-evaluation | frozen baselines and ablations |
| jitter-map | usable CLI composition root |

## Upstream composition

| Ingredient | Executed contribution | Pin |
|---|---|---|
| AWS SDK for Rust | constructs/translates aws_smithy_types retry config | 193882fe11fce9b22424ac913eeb4d03963d5700 |
| RuVector | VectorDB insert/search/get outcome memory | source 5a93328f, crate 2.3.1 |
| RVF | witness creation and verification | rvf-crypto 0.2.0 |
| MetaHarness | bounded Darwin validation and Flywheel replay | source 9ce8b8d, packages 0.10.3/0.1.12 |

## Architecture and evidence

The [specification](docs/specification.md), [architecture](docs/architecture.md), [26 ADRs](docs/adrs/index.md), [12 DDD documents](docs/ddd/index.md), [frozen contract](docs/implementation-contract.md), [research](docs/research.md), and [traceability](docs/traceability.md) define the build. CI results and benchmark receipts are recorded under evidence.

## Frozen benchmark

The deterministic fixture contains 400 requests from eight tenants, a six-tick correlated throttling outage, capacity 25 per tick, seed 20261004, and a 200-tick horizon. All strategies consume the same scenario digest:
`da09384a2793c13f191197de6f4bb4ea15f60e67765c19fd338f4a590f53889d`.

| Strategy | Retry amplification | Successes | p95 ticks | Attempts | Denied retries | Deferred retries |
|---|---:|---:|---:|---:|---:|---:|
| Local full jitter | 5.4375 | 364 | 24 | 2,175 | 0 | 0 |
| AWS standard | 3.0000 | 0 | 0 | 1,200 | 0 | 0 |
| Adaptive shared budget (drop) | 5.3050 | 332 | 24 | 2,122 | 38 | 0 |
| Adaptive deferral | 5.4425 | 329 | 24 | 2,177 | 0 | 55 |

The drop ablation reduced amplification by only 2.4% while losing 8.8% of successful outcomes versus local jitter. The deferral candidate ([ADR-0026](docs/adrs/0026-adaptive-deferral-admission.md)) had zero outright denials on this fixture, rescheduling every budget-exhaustion event instead (55 deferred), but it measured *worse* than the drop ablation: 2.6% higher RAF and 0.9% fewer successes, because deferred retries still spend a fixed `max_attempts` ceiling waiting on contested budget, and some of them ultimately fail anyway. The AWS-standard baseline exhausted its three attempts during the frozen outage. None of the three adaptive/local candidates clears a production-promotion gate; the current decision is **REVISE** for both adaptive strategies. Raw output is in [evidence/benchmark.json](evidence/benchmark.json).

## Maturity and limitations

This is a pre-1.0 evaluation system, not an SDK interceptor, deployment controller, or claim of production superiority. Neither frozen adaptive candidate beats the success-preservation gate. The discrete-event model does not reproduce every network, SDK, or scheduler behavior. RuVector local file storage is evaluation-only; production persistence must remain behind authorized RuVector/Cloud SQL infrastructure. MetaHarness can propose and score policy candidates but cannot promote them. No component may self-promote a policy.
