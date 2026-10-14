# ADR-0108: Checkpoint and replay (`jitter-recovery`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-recovery` to capture a checkpoint and deterministically
replay from it after an interruption, with the planning-table signature
`replay(&Checkpoint, remaining: WorkloadTrace) -> SimulationRun`. The base monolith had
no recovery path at all — a run either completed or was lost.

## Viable alternatives

- Snapshot only the tick number and recompute everything else on resume. Rejected:
  `TenantQueues`'s pending-event schedule and `Budget`'s remaining balance are exactly
  the state a mid-quantum interruption leaves in an intermediate condition; recomputing
  them from scratch would require replaying ticks 0..checkpoint anyway, defeating the
  purpose of checkpointing.
- Snapshot full `TenantQueues` and `Budget` state, and have `replay` take a second wave
  of *new* arrivals via `remaining: WorkloadTrace`, matching the planning-table
  signature literally. Rejected on inspection: every initial arrival is injected into
  `TenantQueues` at tick 0, before any interruption could occur — there is no "second
  wave" of new arrivals left to inject later, because `jitter-workload` only ever
  generates the tick-0 batch (ADR-0102).
- Snapshot full `TenantQueues` and `Budget` state (serde-derived, since both already
  implement it), keep `remaining: WorkloadTrace` as a parameter used for a consistency
  check (its seed must match the checkpoint's) rather than as additional input data.
  Chosen, and documented as a deviation in the crate's own doc comment.

## Decision

`capture(scenario, seed, config, interrupt_after_tick) -> Checkpoint` runs
`jitter-simulation`'s `initial_state` + `drive` with `stop_after_tick =
Some(interrupt_after_tick)`, snapshotting the resulting `TenantQueues`, `Budget`, and
events-so-far. `replay(&Checkpoint, remaining) -> SimulationRun` reconstructs the
identical `policy_for(&checkpoint.config)` and resumes `drive` from the snapshotted
state to completion, then `finalize`s the combined event log.

## Tradeoffs

Not using `remaining` as live input data means the parameter exists for defensive
consistency-checking rather than carrying real information. That is a smaller
compromise than inventing a second wave of arrivals this architecture's own workload
generator (ADR-0102) does not produce.

## Consequences

An interrupted-and-replayed run of `fleet-retry-storm-22`'s shape — 500 tenants, 20
tenants, long-tail weights, interrupted mid-quantum inside the 12-tick outage window —
reaches the exact same `SimulationRun` (and SHA-256 digest via `run_digest`) as an
uninterrupted run. This is the hardest point to resume correctly, because both budget
and queue rotation state are mid-quantum, and it is exactly the point this ADR's test
suite interrupts at.

## Validation

- `cargo test -p jitter-recovery` (4 tests, including
  `interrupted_replay_matches_uninterrupted_digest_at_tick_six` and
  `interruption_deep_inside_outage_window_still_replays_identically` at tick 9)
- `jitter-cli`'s `--interrupt-after`/`--replay` flags exercise this same path through
  the real composition root (manually verified end to end against
  `fixtures/fleet-retry-storm-22.json`: the replayed receipt's three digests matched
  the uninterrupted receipt's byte for byte)
