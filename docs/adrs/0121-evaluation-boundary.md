# ADR-0121: Evaluation boundary (`jitter-evaluation`)

- Status: Accepted
- Date: 2026-10-13
- Implemented: 2026-10-13
- Extends: ADR-0003, ADR-0017, ADR-0019, ADR-0022

## Context

ADR-0027 planned `jitter-evaluation` to score competing policy/budget candidates
without ever implying promotion authority, with a `CandidateEvaluator::score(&self,
&SimulationRun) -> Fitness` trait, and to reproduce a committed fitness-delta ranking
of `WeightedTenantFairBudget` above an unweighted baseline. The base monolith already
had this crate, scoring `jitter_domain::SimulationReport` values; `jitter_domain` no
longer exists after this migration.

## Viable alternatives

- Keep scoring the old `SimulationReport` type. Rejected: not possible after
  `jitter-domain`'s deletion — there is no `SimulationReport` left to score.
- Invent a new scoring formula while migrating. Rejected: the existing formula
  (success rate minus `0.02 * retry_amplification_factor`, `regressed` below 90%
  success) is an already-accepted scoring convention from the base monolith's
  `jitter-candidate` subprocess; changing it mid-migration would be a silent policy
  change, not a structural one.
- Re-point the same formula at `jitter_simulation::SimulationRun`, keep the frozen
  `frozen_scenario`/`fairness_scenario` fixtures byte-identical to the base monolith's
  values, and add `CandidateEvaluator`/`Fitness` as the new typed surface the
  planning table names. Chosen.

## Decision

`SuccessRateMinusAmplification` implements `CandidateEvaluator`, always returning
`authority: "none"`. `run()`/`run_fairness()` reproduce the base monolith's two
frozen-corpus comparisons (`candidates_share_digest_and_never_promote`,
`fairness_ablation_is_deterministic_and_scored`) against the new simulation core. A
new regression test computes the fitness delta between `WeightedTenantFairBudget` and
`TenantFairBudget` on `fairness_scenario` and asserts both the sign (weighted
outranks) and a committed numeric delta, independently re-derived by running the new
code (not assumed).

## Tradeoffs

None — this is a faithful re-pointing of an already-accepted scoring formula at a new
but behaviorally-equivalent simulation core.

## Consequences

Nothing downstream of a `Fitness` value can use it to promote a candidate
automatically; `jitter-adapter-metaharness`'s `FlywheelSink` only ever receives a
`fitness: f64` number inside a `CandidateResult`, never an authority grant.

## Validation

- `cargo test -p jitter-evaluation` (3 tests, including
  `weighted_tenant_fair_outranks_unweighted_baseline_with_committed_delta`)
- `jitter-service`'s `/evaluate` endpoint is a real caller of this crate's
  `CandidateEvaluator`
