# Specification

## Original problem

Independent clients that retry against a shared backend after a correlated failure are
each individually reasonable — and collectively can turn a transient outage into a
sustained one, because every client's backoff curve resets on the same clock. Per-client
retry policy (local jitter, AWS-standard backoff) has no visibility into the other 499
tenants retrying the same backend in the same window, so it cannot prevent this
amplification. JitterMap exists to answer, with evidence rather than assertion: when a
fleet of tenants retries against a shared backend after a correlated failure, which
admission policy keeps the system fair and bounded, and can the claim be proven
deterministically and replayed after a crash?

## Users

- **Platform/SRE engineers** who operate a shared backend (a rate-limited API, a
  database, a queue) and need to choose or validate a fleet-wide retry-admission policy
  before an incident, not during one.
- **Service owners / tenants** behind that shared backend, whose individual retry
  behavior is governed by the chosen policy and whose fairness (not just aggregate
  throughput) is a measured output, not an assumption.
- **Policy researchers / evaluators** who propose competing admission strategies
  (equal-share, demand-weighted, adaptive-drop, adaptive-deferral) and need a frozen,
  replayable benchmark to compare them — with no authority to self-promote their own
  proposal (ADR-0010, ADR-0022).

## Realistic fleet-retry workload

The concrete workload every acceptance criterion below is measured against is
`fleet-retry-storm-22` (docs/architecture.md §4): 500 tenants sharing one backend, a
correlated 12-tick outage window opening at tick 0, a fixed long-tail demand-weight
distribution (a few tenants at weight 20, the rest at weight 1), and a global cap of 64
concurrent retries per tick sourced from AWS RetryConfig. This is deliberately not a toy
two-tenant example: at fleet scale, naive per-client backoff produces exactly the
amplification this system exists to bound, and a long-tail weight distribution is
realistic for multi-tenant SaaS traffic, where a handful of accounts generate most load.

## Input and output

A versioned JSON Scenario contains requests, tenants, optional validated tenant weights,
capacity/outage windows, horizon, seed, failure class, and event ceiling. The CLI selects
local jitter, AWS standard, adaptive shared-budget, adaptive deferral, or tenant-fair
shared-budget policy. Output includes aggregate and per-tenant outcomes, optional Jain
success fairness, nearest RuVector outcomes, exact provenance, RVF root, and authority
none.

## Measurable acceptance criteria

1. Identical input and seed produce byte-stable domain results (determinism; current
   evidence: `replay_is_stable`, docs/traceability.md).
2. Real AWS RetryConfig is translated in integration tests (current evidence:
   `real_retry_config_translates`).
3. Real RuVector insert/search/read-back and RVF verification execute (current evidence:
   `real_vector_round_trip_and_witness`).
4. Invalid, bounded, timeout, persistence, and witness paths are tested.
5. Frozen corpora compare local jitter, AWS standard, adaptive shared-budget, adaptive
   deferral, and tenant-fair admission against exact scenario digests (current evidence:
   `candidates_share_digest_and_never_promote`).
6. No evaluator or memory component can promote a policy (ADR-0010, ADR-0022).
7. **Planned, 22-crate target only.** On `fleet-retry-storm-22`, a kill-mid-run
   interruption at tick 6 followed by `jitter-recovery` replay reaches the same `Receipt`
   digest as an uninterrupted run (docs/architecture.md §7; not yet implemented — see
   docs/implementation-contract.md).
8. **Planned, 22-crate target only.** `jitter-evaluation` reproducibly ranks
   `WeightedTenantFairBudget` above an unweighted baseline on `fleet-retry-storm-22` with
   a committed fitness delta (docs/architecture.md §5, row 21).

## 2026-10-10 proportional-service acceptance criterion

For a validated skewed workload, the system shall expose equal-share and demand-weighted
tenant admission as separate executable policies. The weighted policy must be
deterministic, must not change other policies, and must improve Jain fairness of
per-tenant success ratios over equal-share admission without reducing total successes on
the frozen fixture. Promotion still requires preserving the adaptive-drop success
baseline.

## Interacting bounded contexts

At least three bounded contexts (docs/architecture.md §2, docs/ddd/02-context-map.md)
must interact for any acceptance criterion above to be checkable end to end:

1. **Domain Core / Admission** generates the deterministic workload and admits or defers
   each retry against the shared budget (docs/ddd/03, 04, 06).
2. **Domain Orchestration / Simulation** composes Domain Core into one tick loop and owns
   the interrupted-replay path (docs/ddd/05, 12).
3. **Infrastructure Adapters / Upstream Integration** supplies the real AWS budget,
   persists outcomes to RuVector, and seals the run with an RVF witness (docs/ddd/09).
4. **Evidence** and **Fitness Evaluation** both consume context 2's output: Evidence
   binds the Receipt (docs/ddd/07); Evaluation scores competing policies without
   authority to promote one (docs/ddd/01, ADR-0010).

No criterion above is satisfiable by a single context acting alone — e.g. criterion 7
requires Domain Orchestration (recovery), Infrastructure Adapters (the Ruflo
interruption signal), and Evidence (the digest comparison) together.

## Degradation when integrations are removed

Each ingredient below is a swappable adapter (docs/architecture.md §3), by design — the
domain kernel must keep working with each removed, in a strictly reduced but still
honest way. This is also the basis for the ablation column of
docs/integration-matrix.md.

| Removed | What still works | What degrades or is lost |
|---|---|---|
| **AWS** (RetryConfig source) | Local-jitter and AWS-standard-shaped policies still run against a locally configured budget; domain logic is unaffected (ADR-0005, ADR-0018) | Loses ground truth for the real per-account retry budget; falls back to a configured/default cap instead of a live AWS-sourced one — numbers are no longer traceable to the real backend |
| **RuVector** (outcome memory) | A run still simulates, admits, and produces a `Receipt`; evidence can still be sealed with RVF | Loses nearest-outcome recall and the auditable history of prior runs (ADR-0008); each run becomes a one-off with no comparative memory |
| **RVF** (witness chain) | A run still produces an `OutcomeSummary` and can still be inspected directly | Loses the tamper-evident witness root (ADR-0009); the `Receipt` is no longer independently re-verifiable — only trusted as-reported |
| **Ruflo / MetaHarness** (coordination + Darwin/Flywheel evaluation feedback) | The domain still runs and produces outcomes; `jitter-evaluation`'s scoring math still executes locally (ADR-0023 safety envelope assumes this independence already) | Loses the orchestration-signaled interruption trigger (planned `jitter-adapter-ruflo`) and the Darwin/Flywheel fitness feedback loop; recovery must then be triggered manually and candidate ranking has no advisory second opinion |
| **Demand-weighted scheduling** (`WeightedTenantFairBudget`) | Equal-share `TenantFairBudget` and the adaptive-drop baseline remain fully executable ablations (ADR-0021) | Loses proportional service fairness for skewed demand; on the frozen `fleet-retry-storm-22`-class fixture, Jain fairness regresses from ~0.968 back to ~0.893 (PROJECT_STATUS.md), though adaptive-drop's raw success count is unaffected since it never used weights |

Every row above is an already-exercised ablation path in the current base (ADR-0019,
ADR-0021) or a direct consequence of the hexagonal boundary the planned 22-crate map
enforces (docs/architecture.md §3) — none require new code to reason about; they
describe behavior the architecture already guarantees by construction.
