# Frozen implementation contract

`simulate(&Scenario, &RetryPolicy, &RunControl) -> Result<SimulationReport, DomainError>` is the deterministic core. `evaluate(EvaluateRequest, ports)` validates, resolves policy, simulates, seals, appends memory, and returns an advisory receipt.

All candidates receive the same scenario digest, seed, capacity trace, horizon, ceiling, and metric code. `AdaptiveBudget` drops a retry when shared budget cannot cover its cost. `AdaptiveDeferral` reschedules until projected refill would cover the cost, bounded by the horizon and `MAX_EVENTS`. `TenantFairBudget` preserves drop semantics but orders a same-tick batch through rotating per-tenant FIFO rounds. Optional tenant weights must be positive, cardinality-matched, overflow-safe, and included in the scenario digest. Reports expose per-tenant outcomes and optional Jain success fairness. Eligibility requires publishing the fairness/throughput tradeoff and valid evidence; eligibility is not promotion.
