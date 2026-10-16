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
- local, real AWS RetryConfig, adaptive drop/deferral, and deterministic tenant-fair strategies;
- aggregate and per-tenant RAF, success, denial, deferral, p95, and Jain fairness evidence;
- real RuVector outcome insert/search/read-back;
- real RVF witness-chain sealing;
- bounded Rust process boundaries for Ruflo registry discovery and read-only signed federation sync plus real Darwin numeric evolution and Flywheel replay verification, with append-only receipts and authority none. Ruflo registry and read-only federation-sync reruns auto-create project-local policy anchors; membership and publication remain out of scope.

## Quickstart

    cargo run --locked -p jitter-cli --bin jitter-map -- --scenario examples/correlated-outage.json --policy adaptive --memory .jitter-map-memory
    cargo run --locked -p jitter-cli --bin jitter-map -- --scenario examples/correlated-outage.json --policy adaptive-deferral --memory .jitter-map-memory
    cargo run --locked -p jitter-cli --bin jitter-map -- --scenario examples/skewed-tenant-outage.json --policy tenant-fair-budget --memory .jitter-map-memory
    cargo run --locked -p jitter-cli --bin jitter-map -- --scenario examples/skewed-tenant-outage.json --policy weighted-tenant-fair-budget --memory .jitter-map-memory
    cargo run --locked -p jitter-cli --bin jitter-map -- --scenario fixtures/fleet-retry-storm-22.json --policy weighted-tenant-fair-budget --memory .jitter-map-memory --interrupt-after 6 --checkpoint-out /tmp/checkpoint.json
    cargo run --locked -p jitter-cli --bin jitter-map -- --scenario fixtures/fleet-retry-storm-22.json --policy weighted-tenant-fair-budget --memory .jitter-map-memory --replay /tmp/checkpoint.json
    cargo run --locked -p jitter-evaluation --bin jitter-benchmark
    cargo run --locked -p jitter-evaluation --bin jitter-fairness-benchmark

`--policy` accepts `local`, `aws`, `adaptive`, `a¢ëiºÛkºwµç