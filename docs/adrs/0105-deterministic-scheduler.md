# ADR-0105: Deterministic scheduler (`jitter-scheduler`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-scheduler` to own same-tick admission ordering, combining
policy and budget into a per-tick result, with `TenantQueues` as its aggregate root.
The base monolith used a single global `BinaryHeap<Reverse<Event>>` with two
near-duplicate reordering functions (`fair_order`, `weighted_fair_order`) that differed
only in whether each tenant's quantum was 1 or a configured weight.

## Viable alternatives

- Keep two separate ordering functions (equal-share and weighted). Rejected: weighted
  ordering with all weights set to 1 is mathematically identical to equal-share
  ordering, so maintaining two functions was pure duplication with no behavioral
  difference to justify it.
- Model `TenantQueues` as a persistent per-tenant rotation pointer carried across
  ticks. Rejected: the base monolith's rotation start (`tick % tenants`) is a pure
  function of the tick number, not state carried from the previous round — adding
  persisted rotation state would be new behavior, not a faithful extraction.
- One `weighted_round_robin` function, used for both equal-share and demand-weighted
  ordering (equal-share is just the all-ones-weight-vector case), with `TenantQueues`
  owning the scenario-fixed scheduling parameters (capacity, outage window, horizon)
  alongside the pending-event schedule. Chosen.

## Decision

`TenantQueues` replaces the global heap with a `BTreeMap<tick, Vec<Event>>` grouped by
due tick — equivalent to heap-pop-by-tick, but makes "all events due this tick" a
single lookup instead of a pop-while-peek loop. `drain_tick` first sorts a tick's
batch ascending by `(tenant_id, request_id, attempt)` (reproducing the heap's total
order) and only then applies `weighted_round_robin` when the strategy's
`fair_ordering` flag is set. `schedule_tick(Tick, &mut TenantQueues, &mut Budget, &dyn
RetryPolicy) -> TickResult` is the single entry point combining capacity, policy, and
budget decisions in the same order the base monolith's loop body used them: horizon
check, then capacity, then policy exhaustion, then budget gating/deferral.

## Tradeoffs

Folding capacity/outage/horizon into `TenantQueues` (rather than threading them
through `schedule_tick`'s parameter list) keeps the function signature matching the
planning table's four parameters exactly, at the cost of `TenantQueues` carrying more
state than its name alone suggests. This is the documented reading of "Aggregate Root
(`TenantQueues`)" from the architecture table's DDD reference: it owns the tick-indexed
admission state for the fleet, not just a bare FIFO.

## Consequences

Six new `OutcomeEvent` cases fall out of this decomposition by necessity: `Admitted`,
`Exhausted`, `Deferred`, `Denied`, `HorizonExceeded`, and — the one case with no
counter in the base monolith's `SimulationReport` — `Retried`, covering an ordinary
reschedule that neither succeeds nor terminates. `jitter-outcome`'s `summarize` counts
`Retried` toward `attempts` only, exactly matching the base monolith's unconditional
`attempts += 1` for every non-horizon-exceeded event.

## Validation

- `cargo test -p jitter-scheduler` (8 tests: capacity admission, horizon drop,
  exhaustion, budget denial, deferral exhausting the budget, fair-ordering rotation,
  weighted-quantum ordering, ungated strategies ignoring budget)
- `jitter-simulation`'s golden-value regression test exercises this crate across all
  six strategies against the base monolith's recorded output
