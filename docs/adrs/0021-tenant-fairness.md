# ADR-0021: Deterministic tenant-fair admission

- Status: Accepted
- Date: 2026-10-04
- Implemented: 2026-10-09
- Extended: 2026-10-10

## Context

A shared retry budget can be monopolized by lexical event order. Equal-share round robin corrected that failure, but on demand skew `[8,1,1,1,1,1,1,1]` it over-served small tenants and reduced aggregate success. The policy set therefore needs an explicit distinction between equal-share fairness and proportional service fairness.

## Viable alternatives

- Preserve heap order and only report skew.
- Reserve a static budget slice per tenant.
- Replace equal-share admission with weighted admission.
- Keep equal-share as an ablation and add a distinct demand-weighted policy.

## Decision

Keep `TenantFairBudget` as the equal-share ablation. Add `WeightedTenantFairBudget`, which batches same-tick events into per-tenant FIFO queues, rotates the first tenant by tick, and admits up to each tenant's validated positive scenario weight per round. When weights are absent, it uses equal weights.

Both policies preserve shared-budget drop semantics. Reports expose typed per-tenant outcomes, and Jain fairness is computed over success ratios, not raw success totals. Undefined fairness remains `null`.

## Tradeoffs

Demand weights describe offered load, not entitlement or authorization. Weighted round robin improves proportional service in the frozen skewed workload but can still trail a throughput-first baseline. Tick-local weighting is deterministic and replayable; it is not a production SLA scheduler and does not carry deficit across ticks.

## Consequences

The new policy is opt-in and the equal-share path remains executable for ablation. Darwin, Flywheel, and memory systems remain advisory and cannot promote either policy.

## Validation

- `weighted_policy_tracks_demand_and_improves_service_fairness`
- `weighted_assignment_and_fair_replay_are_deterministic`
- `invalid_weight_boundary_fails_closed`
- `undefined_fairness_is_explicit`
- `fairness_ablation_is_deterministic_and_scored`
- `cargo run --locked -p jitter-evaluation --bin jitter-fairness-benchmark`

On the frozen skewed fixture, weighted admission raised Jain fairness from 0.8932 to 0.9684 versus equal-share admission and increased successes from 63 to 65 at the same 3.7100 RAF. Adaptive drop still produced 72 successes, so promotion remains blocked.
