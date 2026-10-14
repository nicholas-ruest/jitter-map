# ADR-0101: Domain purity boundary (`jitter-scenario`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned a `jitter-scenario` crate owning fleet topology, demand weights, and
the correlated-failure window as validated value objects, with zero dependencies
outside the workspace's domain layer. The base `3db7fd2` monolith mixed `Scenario`,
`validate`, and `scenario_digest` into the same module as the simulation loop, so
"validated" was a convention (call `validate` first) rather than a type-level
guarantee.

## Viable alternatives

- Keep `Scenario` as the only type and re-validate at every call site. Rejected:
  exactly what ADR-0101 exists to stop — a crate could skip validation and no
  compiler error would catch it.
- Validate in place and return the same `Scenario`. Rejected: nothing distinguishes a
  validated from an unvalidated value at the type level.
- Introduce `ValidatedScenario`, constructible only through `validate`, carrying its
  own digest. Chosen.

## Decision

`jitter-scenario` exposes `Scenario` (raw, deserializable, untrusted) and
`ValidatedScenario` (private inner field, only constructible via `validate`).
Every invariant — version, positive counts, horizon exceeding outage, event-limit
bracketing, tenant-weight cardinality/positivity/overflow — is checked exactly once,
at `validate`. `ValidatedScenario::tenant_for_request` is the one place the
demand-weight-cycle assignment logic lives, reused by `jitter-workload`.

## Tradeoffs

A `ValidatedScenario` is slightly more ceremony than a plain struct, but it moves "did
anyone forget to validate?" from a runtime bug class to a type error: nothing
downstream can construct one.

## Consequences

`jitter-scenario` has zero dependencies beyond `serde`/`sha2`/`thiserror` — verified by
the Gate 1 structural checker (`scripts/check_dependency_direction.py`) and by
`cargo tree -p jitter-scenario`.

## Validation

- `cargo test -p jitter-scenario` (12 tests: invariant rejections, digest stability,
  weighted/unweighted tenant assignment)
- `scripts/check_dependency_direction.py` confirms zero non-domain dependency edges
