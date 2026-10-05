# Dependency provenance

| Component | Pin | License | Used behavior |
|---|---|---|---|
| AWS SDK for Rust | source 193882fe11fce9b22424ac913eeb4d03963d5700; crate aws-smithy-types 1.3.4 | Apache-2.0 | RetryConfig standard/max-attempts |
| RuVector | source 5a93328f2fceb0307c25929ed38cd7a0911fdf00; ruvector-core 2.3.1 | MIT | VectorDB insert/search/get |
| RVF | rvf-crypto 0.2.0 | MIT OR Apache-2.0 | witness chain and SHAKE-256 |
| MetaHarness | source 9ce8b8dd89045c3b9a1f809ae58f3589029db4a4 | MIT | bounded Darwin proposal and Flywheel replay evaluation |

Cargo.lock is authoritative for transitive versions. No dependency exists only to inflate the workspace.
