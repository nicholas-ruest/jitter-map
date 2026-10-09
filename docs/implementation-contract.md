# Frozen implementation contract

`simulate(&Scenario, &RetryPolicy, &RunControl) -> Result<SimulationReport, DomainError>` is the deterministic core. `evaluate(EvaluateRequest, ports)` validates, resolves policy, simulates, seals, appends memory, and returns an advisory receipt.

All candidates receive the same scenario digest, seed, capacity trace, horizon, ceiling, and metric code. `AdaptiveBudget` drops a retry to terminal failure the instant the shared budget cannot cover its cost. `AdaptiveDeferral` instead reschedules that retry to the deterministic tick at which projected refill would cover the cost, bounded by the scenario horizon and `MAX_EVENTS`, and falls back to the same drop behavior when refill is structurally impossible (`refill_per_tick == 0`) or would land past the horizon. Either adaptive candidate is eligible only when it preserves success tolerance, reduces RAF, respects fairness and ceilings, and retains valid evidence. Eligibility is not promotion.
