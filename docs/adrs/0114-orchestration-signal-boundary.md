# ADR-0114: Orchestration signal boundary (`jitter-adapter-ruflo`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-adapter-ruflo` as the one adapter with no equivalent in the
base six-package implementation: `HarnessSink`, reporting tick progress and driving
`jitter-recovery`'s interruption trigger. docs/integration-matrix.md records honestly
that no Ruflo client library exists anywhere in this repository — the only Ruflo
surface is an MCP coordination/provenance channel, not a Rust dependency, with no
pinned source revision or package version.

## Viable alternatives

- Vendor or depend on a real Ruflo client crate. Rejected: none exists to depend on;
  pretending otherwise would misrepresent the integration-matrix's own honesty
  constraint ("must not be described as integrated... until jitter-adapter-ruflo
  exists and passes its contract-test suite" — passing the suite requires a real
  `HarnessSink` impl, not a networked Ruflo client that does not exist).
- Build a no-op `HarnessSink` that always reports no interruption. Rejected: this
  would satisfy the trait but could never demonstrate the interruption-triggers-
  recovery path docs/architecture.md §7 requires as this adapter's acceptance
  criterion.
- Build a real, thread-safe interruption-signal primitive
  (`AtomicI64`/`AtomicU64`-backed) that any future Ruflo transport (an MCP listener, a
  webhook) can drive by calling `signal_interruption`, honestly scoped as "not a
  network client, because there is nothing real to speak to yet." Chosen.

## Decision

`RufloSignalAdapter::signal_interruption(at_tick)` / `clear()` / `poll_interruption()`
implement `HarnessSink` with real atomic state, not a hardcoded stub. `on_tick`
increments a real counter (`ticks_seen`), giving a composition root a working report
hook today even with no Ruflo transport behind it yet.

## Tradeoffs

This adapter proves the `HarnessSink` port contract and the interruption-triggers-
recovery wiring, but does not prove any actual Ruflo integration, because none
exists. This gap is carried forward explicitly rather than silently closed.

## Consequences

`jitter-recovery`'s replay path is triggered today by `jitter-cli`'s
`--interrupt-after` flag calling `jitter_recovery::capture` directly, and by this
adapter's `signal_interruption` in `jitter-application`'s
`execute_with_recovery` path — both real callers, neither requiring a live Ruflo
service.

## Validation

- `cargo test -p jitter-adapter-ruflo` (5 tests, including a simulated interruption at
  tick 6 — docs/architecture.md §7's exact tick — and the shared `HarnessSink`
  contract suite)
- `jitter-application`'s `execute_with_recovery_routes_through_checkpoint_replay_on_interruption`
  test drives this adapter's mock-equivalent signal into a real `jitter-recovery`
  replay
