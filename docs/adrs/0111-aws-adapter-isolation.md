# ADR-0111: AWS adapter isolation (`jitter-adapter-aws`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13
- Extends: ADR-0005, ADR-0018

## Context

ADR-0027 planned `jitter-adapter-aws` to isolate the AWS `RetryConfig` anti-corruption
layer (ADR-0005) behind `RetryConfigSource`, standalone from the domain. The base
monolith's `jitter-adapter-aws` already isolated this translation from the domain, but
against the base monolith's own `PolicySource` trait, not `jitter-ports`.

## Viable alternatives

- Keep the existing `PolicySource` trait and signature. Rejected: superseded by
  ADR-0109's `RetryConfigSource`, which is the authoritative port shape for this
  concern now.
- Re-derive the budget-gated/unbounded attempt counts and shared-budget numbers from
  scratch. Rejected: the base monolith's numbers (6 attempts / 1500 shared budget / 18
  refill-per-tick when budget-gated; the real AWS `RetryConfig.max_attempts()` /
  unbounded budget otherwise) are already-accepted values (ADR-0018); changing them
  would be a silent behavior change disguised as a migration.

## Decision

`AwsRetryConfigAdapter::standard(max_attempts)` wraps the real `aws-smithy-types`
`RetryConfig`. `impl RetryConfigSource` reproduces the base monolith's exact
budget-gated/unbounded branching, including the pinned upstream revision string
(`awslabs/aws-sdk-rust@193882fe...`).

## Tradeoffs

None beyond the translation itself — this adapter is a near-direct carry-over, with
only its trait and DTO renamed to match `jitter-ports`.

## Consequences

Zero domain crate references this adapter directly; it is wired only at `jitter-cli`
and `jitter-service`'s composition roots.

## Validation

- `cargo test -p jitter-adapter-aws` (3 tests, including the shared
  `RetryConfigSource` contract suite from `jitter-contract-tests`)
