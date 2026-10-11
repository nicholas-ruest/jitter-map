# Traceability

| Criterion | Context | Crate | ADRs | Tests |
|---|---|---|---|---|
| Deterministic replay | Simulation | jitter-domain | 0002, 0007 | replay_is_stable |
| Fleet admission | Admission | jitter-domain | 0001, 0006, 0021 | budget_limits_amplification, weighted_assignment_and_fair_replay_are_deterministic, invalid_weight_boundary_fails_closed |
| Adaptive deferral | Admission | jitter-domain | 0006, 0026 | deferral_recovers_denied_budget, deferral_fails_closed_without_refill, deferral_replay_is_stable, deferral_fails_closed_beyond_horizon |
| AWS truth | Integration | jitter-adapter-aws | 0004, 0005, 0018 | real_retry_config_translates |
| Auditable memory | Evidence | jitter-adapter-ruvnet | 0008, 0009, 0014 | real_vector_round_trip_and_witness |
| Frozen comparison | Evaluation | jitter-evaluation | 0003, 0017, 0019, 0022 | candidates_share_digest_and_never_promote |
| CLI slice | Workflow | jitter-map | 0024, 0025 | workflow invocation |

| Proportional service fairness | Scenario / Admission Budget / Evaluation | jitter-domain, jitter-evaluation, jitter-map | ADR-0021 | `weighted_policy_tracks_demand_and_improves_service_fairness`; `jitter-fairness-benchmark` | `evidence/fairness-benchmark.json` |
