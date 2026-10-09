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
        max_events: 100000,
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
pub fn run() -> Vec<Row> {
    [
        PolicyKind::LocalJitter,
        PolicyKind::AwsStandard,
        PolicyKind::AdaptiveBudget,
        PolicyKind::AdaptiveDeferral,
    ]
    .into_iter()
    .map(|k| {
        let r: SimulationReport = simulate(&frozen_scenario(), &policy(k), &C).unwrap();
        Row {
            candidate: format!("{k:?}"),
            raf: r.retry_amplification_factor(),
            successes: r.successes,
            p95: r.p95_ticks(),
            attempts: r.attempts,
            denied: r.denied_retries,
            deferred: r.deferred_retries,
            scenario_sha256: r.scenario_sha256,
            authority: "none".into(),
        }
    })
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
}
