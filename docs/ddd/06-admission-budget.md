# Admission context

## Purpose and model

BudgetState never becomes negative. Two admission strategies exist over the same budget model: `AdaptiveBudget` drops an exhausted retry straight to terminal failure, while `AdaptiveDeferral` reschedules it to the tick at which projected refill would cover its cost and only drops it if that tick would fall outside the horizon or refill is structurally impossible (`refill_per_tick == 0`). A denied retry never re-enters the event queue; a deferred retry re-enters it exactly once per deferral decision, strictly bounded by the scenario horizon and `MAX_EVENTS`. Same-tick work is stable by tenant to bound trivial starvation; this is not represented as a weighted SLA scheduler. Aggregate roots protect invariants before state transitions; entities retain stable scenario identity; immutable value objects cross boundaries; domain events are append-only facts, never commands disguised as history.

## Ownership and interfaces

The owning crate is linked from the traceability table. Application ports specify required behavior, while AWS, RuVector, RVF, CLI, and evaluator code remain adapters. No adapter type leaks into `jitter-domain`.

## Invariants

- Authority is always `none`.
- The same frozen input digest is used for every candidate.
- Invalid input fails before mutation.
- Persistence and evidence failures cannot become successful evaluations.

## Domain events

The context emits explicit accepted, rejected, deferred, completed, cancelled, timed-out, or failed facts as appropriate. `deferred_retries` on `SimulationReport` makes the deferred fact typed and countable rather than inferred from attempt counts. IDs are content-derived where evidence identity matters.

## Failure and validation

Unit tests exercise local invariants; application contract tests exercise ports; integration tests call real upstream APIs; the end-to-end CLI covers valid, invalid, timeout, and persistence paths.
