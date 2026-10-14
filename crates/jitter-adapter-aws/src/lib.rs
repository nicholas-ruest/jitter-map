//! Infrastructure Adapter: `AwsRetryConfigAdapter` implements `RetryConfigSource`
//! against the real `aws-smithy-types` `RetryConfig`.
//!
//! ADR-0111 (AWS Adapter Isolation; extends ADR-0005, ADR-0018);
//! docs/ddd/09-upstream-integrations.md. No domain crate names this adapter — it is
//! wired only at a composition root (`jitter-cli`, `jitter-service`).

use aws_smithy_types::retry::RetryConfig;
use jitter_ports::{PolicyProfile, PortError, RetryBudgetSpec, RetryConfigSource};

pub struct AwsRetryConfigAdapter {
    config: RetryConfig,
}

impl AwsRetryConfigAdapter {
    pub fn standard(max_attempts: u32) -> Self {
        Self {
            config: RetryConfig::standard().with_max_attempts(max_attempts),
        }
    }
}

impl RetryConfigSource for AwsRetryConfigAdapter {
    fn retry_budget(&self, profile: PolicyProfile) -> Result<RetryBudgetSpec, PortError> {
        let budget_gated = matches!(profile, PolicyProfile::BudgetGated);
        let max_attempts = if budget_gated {
            6
        } else {
            self.config.max_attempts()
        };
        let (shared_budget, refill_per_tick) = if budget_gated {
            (1500, 18)
        } else {
            (u32::MAX, 0)
        };
        Ok(RetryBudgetSpec {
            max_attempts,
            base_delay: 1,
            cap_delay: 32,
            shared_budget,
            refill_per_tick,
            failure_cost: 1,
            upstream_revision: "awslabs/aws-sdk-rust@193882fe11fce9b22424ac913eeb4d03963d5700"
                .into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_retry_config_translates() {
        let adapter = AwsRetryConfigAdapter::standard(4);
        let spec = adapter.retry_budget(PolicyProfile::Unbounded).unwrap();
        assert_eq!(spec.max_attempts, 4);
        assert!(spec.upstream_revision.contains("193882"));
    }

    #[test]
    fn budget_gated_profile_sources_a_shared_budget() {
        let adapter = AwsRetryConfigAdapter::standard(4);
        let spec = adapter.retry_budget(PolicyProfile::BudgetGated).unwrap();
        assert_eq!(spec.shared_budget, 1500);
        assert_eq!(spec.refill_per_tick, 18);
    }

    #[test]
    fn passes_shared_retry_config_source_contract() {
        jitter_contract_tests::assert_retry_config_source(&AwsRetryConfigAdapter::standard(3));
    }
}
