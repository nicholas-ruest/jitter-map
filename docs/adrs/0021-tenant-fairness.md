# ADR-0021: Deterministic tenant-fair admission

- Status: Accepted
- Date: 2026-10-04
- Implemented: 2026-10-09

## Context

A shared retry budget can be monopolized by lexical event order. The previous implementation contradicted this ADR: its heap key ordered same-tick work by tenant identifier and no `fairness_fixture` existed.

## Viable alternatives

- Preserve heap order and report skew.
- Reserve a static budget slice per tenant.
- Use weighted fair queuing.
- Batch each tick and perform rotating round-robin admission.

## Decision

`PolicyKind::TenantFairBudget` batches all events at a tick into per-tenant FIFO queues, rotates the first tenant by tick, and admits one event per non-empty tenant per round. Existing policies remain unchanged as ablations.

Weighted request mixes are validated and influence assignment only; they do not buy more shared budget. Reports expose typed per-tenant outcomes. Jain fairness is computed over success ratios and is `null` when no tenant succeeds.

## Tradeoffs

Round-robin admission improves equality but can reduce aggregate successes for a dominant tenant. It is deterministic and replayable, but it is not a production SLA scheduler. Evaluation must publish fairness and throughput together.

## Consequences

The policy is opt-in. Darwin, Flywheel, and memory systems remain advisory and cannot promote it.

## Validation

- `weighted_assignment_and_fair_replay_are_deterministic`
- `invalid_weight_boundary_fails_closed`
- `undefined_fairness_is_explicit`
- `fairness_ablation_is_deterministic_and_scored`
- `cargo run --locked -p jitter-evaluation --bin jitter-fairness-benchmark`

On the frozen skewed fixture, Jain fairness increased from 0.1250 to 0.8932 versus adaptive drop, while successes fell from 72 to 63. This is a material tradeoff, not a production-promotion claim.
