# ADR-0106: Domain event log (`jitter-outcome`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-outcome` to own typed result events and per-run aggregation.
The base monolith tracked outcomes as six parallel counters (`attempts`, `successes`,
`terminal_failures`, `denied_retries`, `deferred_retries`, plus per-tenant vectors of
each) mutated inline, with no single typed event representing "what happened to this
attempt."

## Viable alternatives

- Keep parallel counters, now owned by `jitter-scheduler` directly. Rejected: the
  scheduler would have to know aggregation rules (e.g. "denied also counts as
  terminal"), which is exactly the kind of cross-cutting bookkeeping ADR-0027 wanted
  isolated in its own crate.
- A typed `OutcomeEvent` enum with five variants (one per the base monolith's five
  counters). Rejected on inspection: the base monolith's loop has a sixth, uncounted
  case — an attempt that is retried without succeeding, failing, or deferring — which
  only shows up as `attempts += 1` with no other counter incremented.
- A six-variant `OutcomeEvent` enum (`Admitted`, `Retried`, `Deferred`, `Denied`,
  `Exhausted`, `HorizonExceeded`) plus a pure `summarize` aggregation function. Chosen.

## Decision

`summarize(&[OutcomeEvent], tenants) -> OutcomeSummary` is infallible and the sole
place aggregation rules live: `HorizonExceeded` counts toward `terminal_failures` but
not `attempts` (a request dropped beyond the horizon was never actually attempted
against the backend); every other variant counts toward `attempts`; `Denied` counts
toward both `denied_retries` and `terminal_failures`. `AdmissionStrategy` — the
six-variant strategy identifier recorded on every summary — lives in this crate too,
because it is report metadata, not simulation-loop state.

## Tradeoffs

Adding a sixth event variant with no counterpart in the base `SimulationReport` is a
deliberate expansion of the typed surface, not a minimal extraction. It is the only
way to keep `summarize`'s accounting exact without smuggling an implicit
"uncategorized attempt" case into the aggregation logic.

## Consequences

`jitter-outcome` has zero dependencies beyond `serde`. It is depended on by every
layer above Domain Core — `jitter-scheduler`, `jitter-simulation`,
`jitter-recovery`, `jitter-evidence`, `jitter-ports`, `jitter-evaluation` — making its
event vocabulary the one place "what can happen to a retry attempt" is defined.

## Validation

- `cargo test -p jitter-outcome` (11 tests: each event category's aggregation effect,
  zero-event tenant slots, serde round-trip exactness, Jain-fairness edge cases,
  strategy-flag semantics)
