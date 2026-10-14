# ADR-0116: Evidence receipt model (`jitter-evidence`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13

## Context

ADR-0027 planned `jitter-evidence` to formalize the ad hoc evidence-binding behavior
the base monolith's `EvaluationReceipt` already implied (commits #8, #10), as a typed,
pure `bind` function. The planning table shows `bind`'s inputs including a
`WitnessRef` (a `jitter-ports` type).

## Viable alternatives

- Depend on `jitter-ports` for `WitnessRef` directly. Rejected: docs/architecture.md
  §3's diagram draws `EVID --> OUT` only, no edge to `PORTS` — the same
  structurally-checked rule ADR-0109 already applied to keep `jitter-ports` from
  depending outward applies in the other direction here too.
- Take the witness digest as a plain `String` instead of a typed `WitnessRef`. Chosen,
  documented as a deviation in the crate's own doc comment — `jitter-application`,
  which depends on both `jitter-ports` and `jitter-evidence`, narrows `WitnessRef` to
  its digest string before calling `bind`.

## Decision

`bind(scenario_digest, original_requests, tenant_original_requests, &OutcomeSummary,
witness_digest) -> Receipt` is pure. `outcome_digest` is a SHA-256 of the summary's
canonical JSON serialization, computed by `outcome_summary_digest` — a free function
any later holder of the same `OutcomeSummary` can call to re-verify the receipt
without re-running the simulation.

## Tradeoffs

A `Receipt` is never mutated in place, only superseded by binding a new one from a new
summary — enforced by convention (no mutator methods exist on `Receipt`), not by a
private-field trick, since every field is legitimately read by downstream consumers
(the CLI prints the whole struct as JSON).

## Consequences

`authority` is always the literal string `"none"` — no evaluator or memory component
reachable from this receipt can promote a policy (ADR-0010, ADR-0022).

## Validation

- `cargo test -p jitter-evidence` (5 tests: authority is always none, outcome digest
  is stable and independently re-derivable, outcome digest changes when the summary
  changes, two receipts differing only in witness digest share the same outcome
  digest, serde round-trip exactness)
