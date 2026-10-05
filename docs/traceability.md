# Traceability

| Criterion | Context | Crate | ADRs | Tests |
|---|---|---|---|---|
| Deterministic replay | Simulation | jitter-domain | 0002, 0007 | replay_is_stable |
| Fleet admission | Admission | jitter-domain | 0001, 0006, 0021 | budget_limits_amplification |
| AWS truth | Integration | jitter-adapter-aws | 0004, 0005, 0018 | real_retry_config_translates |
| Auditable memory | Evidence | jitter-adapter-ruvnet | 0008, 0009, 0014 | real_vector_round_trip_and_witness |
| Frozen comparison | Evaluation | jitter-evaluation | 0003, 0017, 0019, 0022 | candidates_share_digest_and_never_promote |
| CLI slice | Workflow | jitter-map | 0024, 0025 | workflow invocation |
