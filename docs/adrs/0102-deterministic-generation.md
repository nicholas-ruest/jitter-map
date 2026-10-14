# ADR-0102: Deterministic generation (`jitter-workload`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-workload` to generate the retry request trace deterministically
from a scenario and a seed. In the base monolith, the tick-0 arrival of every request was
generated inline inside `simulate`, interleaved with the admission loop itself.

## Viable alternatives

- Generate the *entire* retry trace (including every future retry) up front. Rejected:
  a retry's timing depends on budget/policy decisions made during the run — it cannot
  be known before the run happens, so "the trace" can only mean the initial arrivals.
- Generate initial arrivals without carrying the seed forward. Rejected: the seed is
  still needed downstream for the jitter delay computation in `jitter-policy`, and
  dropping it here would force a second, separate plumbing path for the same value.
- Generate initial arrivals and attach the seed to the resulting `WorkloadTrace`.
  Chosen.

## Decision

`generate(&ValidatedScenario, seed) -> WorkloadTrace` produces one `RetryRequest{
request_id, tenant_id, tick: 0, attempt: 1}` per request, tenant-assigned via
`ValidatedScenario::tenant_for_request`. The `WorkloadTrace` carries `seed` so
`jitter-simulation` can thread the same seed into `jitter-policy`'s jitter computation
without a second source of truth.

## Tradeoffs

The initial trace itself does not depend on `seed` (tenant assignment is a pure
function of the demand-weight cycle), so the seed parameter is, on its own, inert for
this function's own output. It is kept because the single-seed-per-run invariant
(ADR-0007) is easier to guarantee by always carrying the seed on the trace than by
re-deriving "the seed for this run" in two different crates.

## Consequences

Determinism is the whole contract: `generate` is infallible given a `ValidatedScenario`.

## Validation

- `cargo test -p jitter-workload` (4 tests, including
  `determinism_same_seed_same_trace_bytes`: two independent calls with the same seed
  produce byte-identical serialized output)
