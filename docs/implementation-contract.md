# Frozen implementation contract

`simulate(&Scenario, &RetryPolicy, &RunControl) -> Result<SimulationReport, DomainError>` is the deterministic core. `evaluate(EvaluateRequest, ports)` validates, resolves policy, simulates, seals, appends memory, and returns an advisory receipt.

All candidates receive the same scenario digest, seed, capacity trace, horizon, ceiling, and metric code. Adaptive is eligible only when it preserves success tolerance, reduces RAF, respects fairness and ceilings, and retains valid evidence. Eligibility is not promotion.
