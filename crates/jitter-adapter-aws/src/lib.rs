use aws_smithy_types::retry::RetryConfig;
use jitter_application::PolicySource;
use jitter_domain::{PolicyKind, RetryPolicy};
use std::collections::BTreeMap;
pub struct AwsPolicySource {
    config: RetryConfig,
}
impl AwsPolicySource {
    pub fn standard(max_attempts: u32) -> Self {
        Self {
            config: RetryConfig::standard().with_max_attempts(max_attempts),
        }
    }
}
impl PolicySource for AwsPolicySource {
    fn policy(&self, kind: PolicyKind) -> Result<RetryPolicy, String> {
        let budget_gated = matches!(
            kind,
            PolicyKind::AdaptiveBudget
                | PolicyKind::AdaptiveDeferral
                | PolicyKind::TenantFairBudget
        );
        let attempts = if budget_gated {
            6
        } else {
            self.config.max_attempts()
        };
        let (budget, refill) = if budget_gated {
            (1500, 18)
        } else {
            (u32::MAX, 0)
        };
        Ok(RetryPolicy {
            kind,
            max_attempts: attempts,
            base_delay: 1,
            cap_delay: 32,
            shared_budget: budget,
            refill_per_tick: refill,
            failure_costs: BTreeMap::from([
                ("transient".into(), 1),
                ("timeout".into(), 2),
                ("throttling".into(), 1),
                ("permanent".into(), u32::MAX),
            ]),
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
        let s = AwsPolicySource::standard(4);
        let p = s.policy(PolicyKind::AwsStandard).unwrap();
        assert_eq!(p.max_attempts, 4);
        assert!(p.upstream_revision.contains("193882"));
    }
}
