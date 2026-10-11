# Scenario aggregate

## Purpose and model

Scenario owns requests, tenants, optional positive tenant weights, capacity and outage windows, horizon, seed, and event ceiling. It rejects nonpositive counts, weight/cardinality mismatches, overflow, invalid horizons, unsupported versions, and unsafe event limits before allocation. Weighted assignment is deterministic and becomes part of the scenario digest. Aggregate roots protect invariants before state transitions; entities retain stable scenario identity; immutable value objects cross boundaries; domain events are append-only facts.

## Ownership and interfaces

The owning crate is linked from the traceability table. Application ports specify required behavior, while AWS, RuVector, RVF, CLI, and evaluator code remain adapters. No adapter type leaks into `jitter-domain`.

## Invariants

- Authority is always `none`.
- The same frozen input digest is used for every candidate.
- Invalid input fails before mutation.
- Persistence and evidence failures cannot become successful evaluations.

## Domain events

The context emits explicit accepted, rejected, completed, cancelled, timed-out, or failed facts as appropriate. IDs are content-derived where evidence identity matters.

## Failure and validation

Unit tests exercise local invariants; application contract tests exercise ports; integration tests call real upstream APIs; the end-to-end CLI covers valid, invalid, timeout, and persistence paths.

## Demand weights

`tenant_weights` represent offered-load proportions. They are positive, length-matched to the tenant count, and sum-bounded at the scenario boundary. They are neither authorization nor SLA entitlement. The same value object drives deterministic request assignment and the explicit weighted-admission policy.
