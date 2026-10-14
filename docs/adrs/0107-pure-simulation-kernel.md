# ADR-0107: Pure simulation kernel (`jitter-simulation`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-simulation` to compose the six Domain Core crates behind one
deterministic entrypoint, with zero I/O, clock, or thread dependency, structurally
enforced. The planning table's signature for this entrypoint is `run(&ValidatedScenario,
seed: u64) -> SimulationRun`.

## Viable alternatives

- Implement `run` with exactly the two-parameter signature shown in the planning
  table. Rejected on inspection: a bare scenario and seed cannot determine which of
  the six admission strategies to run or what budget it has — those are
  adapter-sourced inputs (ADR-0005, this ADR's own composition), not scenario fields.
  A `run` that could only ever run one hardcoded strategy would defeat the entire
  point of a system built to compare strategies.
- Add strategy/budget fields directly onto `Scenario`. Rejected: that would make
  budget parameters (which legitimately vary by AWS account, by environment) part of
  the *scenario* digest, conflating "what workload are we testing" with "what
  infrastructure budget is available," which ADR-0011 already treats as a separate
  evidence-bound input.
- Add a third parameter, `AdmissionConfig` (strategy + resolved delay/budget
  parameters), keeping `run` infallible and I/O/clock-free — the two load-bearing
  parts of the original contract. Chosen, and documented as a deviation in the
  crate's own doc comment.
- Separately: return `DomainError::EventLimit` when `attempts >= max_events`, exactly
  matching the base monolith. Rejected: `run`'s contract requires infallibility; an
  `Err` return is incompatible with that. Instead, `drive` stops admitting once the
  counted-attempt ceiling is reached, truncating deterministically rather than
  erroring. Documented as a divergence that no fixture in this workspace actually
  exercises (every configured `max_events` is far above what `fleet-retry-storm-22`
  or any test scenario reaches).

## Decision

`run` composes `jitter-workload::generate`, builds `TenantQueues`/`Budget` via
`initial_state`, selects `ExponentialJitter` or `DemandWeighted` via `policy_for`, and
drives the tick loop via `drive` to `finalize`. `initial_state`, `policy_for`, `drive`,
and `finalize` are each public so `jitter-recovery` can reuse them to build and resume
from a checkpoint without duplicating the kernel's composition logic.

## Tradeoffs

Splitting `run` into four public composable pieces instead of one opaque function
trades a slightly larger public surface for `jitter-recovery` never having to
re-implement "how is the initial state built" or "how does the tick loop advance" —
exactly the duplication that would make an interrupted-vs-uninterrupted digest
mismatch possible to introduce by accident.

## Consequences

`jitter-simulation`'s `Cargo.toml` names only Domain Core crates plus `serde` —
checked by `scripts/check_dependency_direction.py` (Gate 1) and confirmed by `cargo
tree -p jitter-simulation`.

## Validation

- `cargo test -p jitter-simulation` (5 tests, including
  `matches_base_monolith_golden_values_across_every_strategy`: this exact
  scenario+policy pairing was verified side by side against `jitter_domain::simulate`
  — including full per-tenant and `completed_at` comparison — before `jitter-domain`
  was deleted; the golden values frozen here are that verified-equal result)
- 3x-repeated determinism is implied by `run_is_deterministic_across_repeated_calls`
