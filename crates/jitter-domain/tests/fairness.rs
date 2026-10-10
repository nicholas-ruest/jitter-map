use jitter_domain::{FailureClass, PolicyKind, RetryPolicy, RunControl, Scenario, simulate};
use std::collections::BTreeMap;

struct Control;
impl RunControl for Control {
    fn cancelled(&self) -> bool {
        false
    }
    fn deadline_exceeded(&self) -> bool {
        false
    }
}

fn scenario() -> Scenario {
    Scenario {
        version: 1,
        requests: 150,
        tenants: 3,
        capacity_per_tick: 5,
        outage_ticks: 4,
        horizon_ticks: 100,
        seed: 20261009,
        failure: FailureClass::Throttling,
        tenant_weights: Some(vec![8, 1, 1]),
        max_events: 10_000,
    }
}

fn policy(max_attempts: u32) -> RetryPolicy {
    RetryPolicy {
        kind: PolicyKind::TenantFairBudget,
        max_attempts,
        base_delay: 1,
        cap_delay: 16,
        shared_budget: 120,
        refill_per_tick: 4,
        failure_costs: BTreeMap::from([("throttling".into(), 1)]),
        upstream_revision: "test".into(),
    }
}

#[test]
fn weighted_assignment_and_fair_replay_are_deterministic() {
    let first = simulate(&scenario(), &policy(5), &Control).unwrap();
    let second = simulate(&scenario(), &policy(5), &Control).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.tenant_original_requests, vec![120, 15, 15]);
    assert_eq!(first.tenant_attempts.len(), 3);
    assert_eq!(first.tenant_successes.len(), 3);
    assert!(first.tenant_success_fairness().is_some());
}

#[test]
fn invalid_weight_boundary_fails_closed() {
    let mut invalid = scenario();
    invalid.tenant_weights = Some(vec![1, 0, 1]);
    assert!(simulate(&invalid, &policy(5), &Control).is_err());

    invalid.tenant_weights = Some(vec![1, 1]);
    assert!(simulate(&invalid, &policy(5), &Control).is_err());
}

#[test]
fn undefined_fairness_is_explicit() {
    let report = simulate(&scenario(), &policy(1), &Control).unwrap();
    assert_eq!(report.successes, 0);
    assert_eq!(report.tenant_success_fairness(), None);
}
