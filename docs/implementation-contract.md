# Frozen implementation contract

## Current API contract (base `3db7fd2`, implemented today)

`simulate(&Scenario, &RetryPolicy, &RunControl) -> Result<SimulationReport, DomainError>`
is the deterministic core. `evaluate(EvaluateRequest, ports)` validates, resolves policy,
simulates, seals, appends memory, and returns an advisory receipt.

All candidates receive the same scenario digest, seed, capacity trace, horizon, ceiling,
and metric code. `AdaptiveBudget` drops a retry when shared budget cannot cover its cost.
`AdaptiveDeferral` reschedules until projected refill would cover the cost, bounded by
the horizon and `MAX_EVENTS`. `TenantFairBudget` preserves drop semantics but orders a
same-tick batch through rotating per-tenant FIFO rounds. Optional tenant weights must be
positive, cardinality-matched, overflow-safe, and included in the scenario digest.
Reports expose per-tenant outcomes and optional Jain success fairness. Eligibility
requires publishing the fairness/throughput tradeoff and valid evidence; eligibility is
not promotion.

### Weighted fairness extension

`WeightedTenantFairBudget` is opt-in. With `tenant_weights`, each same-tick round admits
at most the tenant's weight; without weights, every quantum is one. Inputs must pass the
shared scenario validator. Equal-share, adaptive drop and adaptive deferral remain
executable ablations. All receipts retain `authority: none`.

## Planned API contract (22-crate target, docs/architecture.md Â§5 â€” not yet built)

Each row below is the typed public API a crate must expose once it is extracted; a crate
is not compliant if its API drifts from this table without a new ADR. Full detail lives
in docs/architecture.md Â§5; this contract restates only the function/type signatures and
their failure semantics.

| Crate | Entry point | Failure contract |
|---|---|---|
| `jitter-scenario` | `validate(Scenario) -> Result<ValidatedScenario, ScenarioError>` | Rejects invalid input before any allocation; never panics on malformed input |
| `jitter-workload` | `generate(&ValidatedScenario, seed: u64) -> WorkloadTrace` | Infallible given a `ValidatedScenario`; determinism is the contract, not error handling |
| `jitter-policy` | `RetryPolicy::next_delay(&self, &AttemptHistory) -> Decision` | Pure function; no I/O, no error variant â€” a policy that cannot decide is a bug, not a `Result` |
| `jitter-budget` | `Budget::try_consume(&mut self, Weight) -> Result<(), BudgetExhausted>` | `BudgetExhausted` is the only failure mode; budget state never goes negative |
| `jitter-scheduler` | `schedule_tick(Tick, &mut TenantQueues, &mut Budget, &dyn RetryPolicy) -> TickResult` | Infallible given valid state; invalid state is a caller contract violation, not a runtime error |
| `jitter-outcome` | `summarize(&[OutcomeEvent]) -> OutcomeSummary` | Infallible; serde round-trip must be exact |
| `jitter-simulation` | `run(&ValidatedScenario, seed: u64) -> SimulationRun` | Infallible given valid inz.¶›­¶»§q«^