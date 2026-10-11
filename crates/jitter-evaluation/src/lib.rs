use jitter_domain::{
    FailureClass, PolicyKind, RetryPolicy, RunControl, Scenario, SimulationReport, simulate,
};
use serde::Serialize;
use std::collections::BTreeMap;
struct C;
impl RunControl for C {
    fn cancelled(&self) -> bool {
        false
    }
    fn deadline_exceeded(&self) -> bool {
        false
    }
}
#[derive(Serialize)]
pub struct Row {
    pub candidate: String,
    pub raf: f64,
    pub successes: u32,
    pub p95: u32,
    pub attempts: u64,
    pub denied: u64,
    pub deferred: u64,
    pub tenant_success_fairness: Option<f64>,
    pub scenario_sha256: String,
    pub authority: String,
}
pub fn frozen_scenario() -> Scenario {
    Scenario {
        version: 1,
        requests: 400,
        tenants: 8,
        capacity_per_tick: 25,
        outage_ticks: 6,
        horizon_ticks: 200,
        seed: 20261004,
        failure: FailureClass::Throttling,
        tenant_weights: None,
        max_events: 100000,
    }
}
pub fn fairness_scenario() -> Scenario {
    Scenario {
        version: 1,
        requests: 600,
        tenants: 8,
        capacity_per_tick: 20,
        outage_ticks: 8,
        horizon_ticks: 240,
        seed: 20261009,
        failure: FailureClass::Throttling,
        tenant_weights: Some(vec![8, 1, 1, 1, 1, 1, 1, 1]),
        max_events: 150000,
    }
}
fn policy(kind: PolicyKind) -> RetryPolicy {
    RetryPolicy {
        kind,
        max_attempts: if kind == PolicyKind::AwsStandard {
            3
        } else {
            6
        },
        base_delay: 1,
        cap_delay: 32,
        shared_budget: 1500,
        refill_per_tick: 18,
        failure_costs: BTreeMap::from([("throttling".into(), 1)]),
        upstream_revision: if kind == PolicyKind::AwsStandard {
            "aws-sdk-rust@193882fe"
        } else {
            "jitter-map@frozen"
        }
        .into(),
    }
}
fn row_for(scenario: &Scenario, kind: PolicyKind) -> Row {
    let report: SimulationReport = simulate(scenario, &policy(kind), &C).unwrap();
    Row {
        candidate: format!("{kind:?}"),
        raf: report.retry_amplification_factor(),
        successes: report.successes,
        p95: report.p95_ticks(),
        attempts: report.attempts,
        denied: report.denied_retries,
        deferred: report.deferred_retries,
        tenant_success_fairness: report.tenant_success_fairness(),
        scenario_sha256: report.scenario_sha256,
        authority: "none".into(),
    }
}
pub fn run() -> Vec<Row> {
    [
        PolicyKind::LocalJitter,
        PolicyKind::AwsStandard,
        PolicyKind::AdaptiveBudget,
        PolicyKind::AdaptiveDeferral,
        PolicyKind::TenantFairBudget,
        PolicyKind::WeightedTenantFairBudget,
    ]
    .into_iter()
    .map(|kind| row_for(&frozen_scenario(), kind))
    .collect()
}
pub fn run_fairness() -> Vec<Row> {
    [
        PolicyKind::AdaptiveBudget,
        PolicyKind::AdaptiveDeferral,
        PolicyKind::TenantFairBudget,
        PolicyKind::WeightedTenantFairBudget,
    ]
    .into_iter()
    .map(|kind| row_for(&fairness_scenario(), kind))
    .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn candidates_share_digest_and_never_promote() {
        let r = run();
        assert!(r.iter().all(|x| x.scenario_sha256 == r[0].scenario_sha256));
        assert!(r.iter().all(|x| x.authority == "none"));
        assert!(r[2].raf < r[0].raf);
        assert!(r[3].deferred > 0);
    }
    #[test]
    fn fairness_ablation_is_deterministic_and_scored() {
        let first = run_fairness();
        let second = run_fairness();
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
        assert!(
            first
                .iter()
                .all(|row| row.tenant_success_fairness.is_some())
        );
    }
}
