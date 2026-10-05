# Dream Machine build — 2026-10-04 — JitterMap

JitterMap is a Rust fleet-retry admission evaluator. It measures what client-local backoff misses: how independent retries combine during one correlated failure.

## Implemented composition

- AWS SDK for Rust `RetryConfig` translation at revision `193882fe11fce9b22424ac913eeb4d03963d5700`.
- RuVector `VectorDB` outcome insert, search and readback.
- RVF cryptographic witness chain creation, verification and tamper rejection.
- MetaHarness Darwin numeric candidates and Flywheel signed replay, both advisory-only.

The workspace contains domain, application, AWS adapter, Ruvnet adapter, CLI/service and evaluation crates. A real CLI reads a typed scenario, evaluates a policy, persists evidence through RuVector, seals it with RVF and prints an inspectable JSON receipt.

## Architecture

https://github.com/nicholas-ruest/jitter-map/blob/main/docs/assets/jitter-map-overview.svg

The repository contains 25 decision-specific ADRs and 12 detailed DDD documents, with traceability from acceptance criteria to crates and tests.

## Reproducible validation

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-targets --all-features
cargo deny check
cargo run --locked -p jitter-evaluation --bin jitter-benchmark
```

Formatting, strict Clippy, workspace tests, dependency/security policy and the benchmark executed successfully on the governed Rust 1.99.0 runtime and in GitHub CI. The implementation was squash-merged at `72d792bfd7689b91185376f0f28213824d352b06`; the final validation commit is recorded in the repository evidence receipt.

## Result and limitation

On the frozen fixture, adaptive admission lowered RAF from 5.4375 to 5.3050 but reduced successes from 364 to 332. This is not a superiority claim and does not clear promotion. Darwin and Flywheel may propose or score candidates; they cannot promote them. Production Cloud SQL/RuVector and live service traces remain unvalidated.

Repository: https://github.com/nicholas-ruest/jitter-map

Research report filename: `dream-machine-research-2026-10-04-jitter-map.md`

Decision: REVISE.

This announcement contains no secrets, credentials, private source, personal data or confidential material.
