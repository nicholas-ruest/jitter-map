# ADR-0122: Shared contract test suite (`jitter-contract-tests`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-contract-tests` as a dev-only shared test suite, one
`assert_<port>` function per `jitter-ports` trait, appearing only under
`[dev-dependencies]` workspace-wide, and gating all adapter extraction (Phase 0: this
crate must compile against `jitter-ports`' mocks before any adapter is built).

## Viable alternatives

- Duplicate the same assertions inside every adapter's own test module. Rejected:
  this is exactly the "could be gamed by hollow packages" risk ADR-0027's rationale
  names — five independent copies of "does `write` produce a stable digest" could
  silently drift out of sync.
- Give `jitter-contract-tests` a dev-dependency on `jitter-ports` so it could also
  validate `jitter-ports`' own mocks via the same assertion functions. Rejected: this
  creates a dev-dependency cycle (`jitter-ports` dev-deps on `jitter-contract-tests`,
  which regular-deps on `jitter-ports`) that Cargo resolves by compiling
  `jitter-ports` twice in one test binary, producing two distinct compiled instances
  of the same trait — the mock's `impl` and the assertion function's bound fail to
  type-check against each other. See ADR-0109's Decision section for the fix
  (`jitter-ports`' mocks are tested directly, with no dependency on this crate).
- One `assert_*` function per port trait, generic over any implementation, imported
  only from adapter crates' `[dev-dependencies]`. Chosen.

## Decision

Five functions — `assert_retry_config_source`, `assert_witness_sink`,
`assert_memory_sink`, `assert_harness_sink`, `assert_flywheel_sink` — each
constructing representative sample inputs and asserting the invariants that make an
implementation honest (stability for identical input, a changing digest for changing
input, non-empty provenance strings, growing nearest-prior recall, no panics). Every
one of the five real adapters invokes its corresponding function from its own
`#[cfg(test)]` suite.

## Tradeoffs

None beyond the dev-dependency-cycle lesson already recorded in ADR-0109.

## Consequences

`scripts/check_dependency_direction.py` (the Gate 1 structural checker) verifies this
crate never appears outside `[dev-dependencies]` anywhere in the workspace, and that
no dependency cycle exists among the 22 crates' normal (non-dev) edges.

## Validation

- All 5 real adapters (`jitter-adapter-aws`, `-rvf`, `-ruvector`, `-ruflo`,
  `-metaharness`) invoke and pass their contract function in their own test suites
- `python3 scripts/check_dependency_direction.py` passes Gate 1 for all 22 crates
