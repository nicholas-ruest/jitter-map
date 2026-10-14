//! Fitness Evaluation: scores competing admission-strategy candidates against the
//! frozen `fleet-retry-storm-22`-class fixtures, without ever implying promotion
//! authority.
//!
//! ADR-0121 (Evaluation Boundary; extends ADR-0003, ADR-0017, ADR-0019, ADR-0022);
//! docs/ddd/01-ubiquitous-language.md (Promotion Eligibility). Kept as its own
//! bounded context so scoring never pollutes `jitter-policy`.

use jitter_outcome::AdmissionStrategy;
use jitter_scenario::{FailureClass, Scenario, ValidatedScenario, validate};
use jitter_simulation::{AdmissionConfig, SimulationRun};
use serde::Serialize;

fn config_for(strategy: AdmissionStrategy) -> AdmissionConfig {
    AdmissionConfig {
        strategy,
        max_attempts: if strategy == AdmissionStrategy::AwsStandard {
            3
        } else {
            6
        },
        base_delay: 1,
        cap_delay: 32,
        shared_budget: 1500,
        refill_per_tick: 18,
        failure_cost: 1,
    }
}

pub fn frozen_scenario() -> ValidatedScenario {
    validate(Scenario {
        version: 1,
        requests: 400,
        tenants: 8,
        capacity_per_tick: 25,
        outage_ticks: 6,
        horizon_ticks: 200,
        seed: 20261004,
        failure: FailureClass::Throttling,
        tenant_weights: None,
        max_events: 100_000,
    })
    .expect("frozen_scenario is a checked-in valid fixture")
}

pub fn fairness_scenario() -> ValidatedScenario {
    validate(Scenario {
        version: 1,
        requests: 600,
        tenants: 8,
        capacity_per_tick: 20,
        outage_ticks: 8,
        horizon_ticks: 240,
        seed: 20261009,
        failure: FailureClass::Throttling,
        tenant_weights: Some(vec![8, 1, 1, 1, 1, 1, 1, 1]),
        max_events: 150_000,
    })
    .expect("fairness_scenario is a checked-in valid fixture")
}

#[derive(Debug, Clone, Serialize)]
pub struct Row {
    pub candidate: String,
    pub raf: f64,
    pub successes: u32,
    pub p95: u32,
    pub attempts: u64,
    pub denied: u64,
    pub deferred: u64,
    pub tenant_success_fairness: Option<f64>,
    pub scenario_digest: String,
    pub authority: String,
}

fn row_for(scenario: &ValidatedScenario, strategy: AdmissionStrategy) -> Row {
    let run = jitter_simulation::run(scenario, scenario.scenario().seed, &config_for(strategy));
    Row {
        candidate: format!("{strategy:?}"),
        raf: run.retry_amplification_factor(),
        successes: run.summary.successes,
        p95: run.p95_ticks(),
        attempts: run.summary.attempts,
        denied: run.summary.denied_retries,
        deferred: run.summary.deferred_retries,
        tenant_success_fairness: run.tenant_success_fairness(),
        scenario_digest: run.scenario_digest.clone(),
        authority: "none".into(),
    }
}

pub fn run() -> Vec<Row> {
    [
        AdmissionStrategy::LocalJitter,
        AdmissionStrategy::AwsStandard,
        AdmissionStrategy::AdaptiveBudget,
        AdmissionStrategy::AdaptiveDeferral,
        AdmissionStrategy::TenantFairBudget,
        AdmissionStrategy::WeightedTenantFairBudget,
    ]
    .into_iter()
    .map(|strategy| row_for(&frozen_scenario(), strategy))
    .collect()
}

pub fn run_fairness() -> Vec<Row> {
    [
        AdmissionStrategy::AdaptiveBudget,
        AdmissionStrategy::AdaptiveDeferral,
        AdmissionStrategy::TenantFairBudget,
        AdmissionStrategy::WeightedTenantFairBudget,
    ]
    .into_iter()
    .map(|strategy| row_for(&fairness_scenario(), strategy))
    .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Fitness {
    pub primary: f64,
    pub regressed: bool,
    pub authority: &'static str,
}

/// Scores a completed run. Never returns a `Fitness` that implies promotion
/// authority (ADR-0010, ADR-0022) — `authority` is always `"none"`, and nothing
/// that reads a `Fitness` can use it to promote a candidate automatically.
pub trait CandidateEvaluator {
    fn score(&self, run: &SimulationRun) -> Fitness;
}

/// The default evaluator: success rate penalized by retry amplification. Mirrors the
/// scoring formula the base `3db7fd2` monolith used in its MetaHarness candidate
/// scorer (`jitter-candidate`), now expressed against `SimulationRun` instead of the
/// old `SimulationReport`.
pub struct SuccessRateMinusAmplification;

impl CandidateEvaluator for SuccessRateMinusAmplification {
    fn score(&self, run: &SimulationRun) -> Fitness {
        let success_rate = f64::from(run.summary.successes) / f64::from(run.original_requests);
        let raf = run.retry_amplification_factor();
        Fitness {
            primary: success_rate - 0.02 * raf,
            regressed: success_rate < 0.9,
            authority: "none",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidates_share_digest_and_never_promote() {
        let rows = run();
        assert!(
            rows.iter()
                .all(|row| row.scenario_digest == rows[0].scenario_digest)
        );
        assert!(rows.iter().all(|row| row.authority == "none"));
        assert!(rows[2].raf < rows[0].raf);
        assert!(rows[3].deferred > 0);
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

    /// Row 21's acceptance criterion: evaluation reproducibly ranks
    /// `WeightedTenantFairBudget` above an unweighted baseline with a committed
    /// fitness delta.
    #[test]
    fn weighted_tenant_fair_outranks_unweighted_baseline_with_committed_delta() {
        let scenario = fairness_scenario();
        let baseline = jitter_simulation::run(
            &scenario,
            scenario.scenario().seed,
            &config_for(AdmissionStrategy::TenantFairBudget),
        );
        let weighted = jitter_simulation::run(
            &scenario,
            scenario.scenario().seed,
            &config_for(AdmissionStrategy::WeightedTenantFairBudget),
        );
        let evaluator = SuccessRateMinusAmplification;
        let baseline_fitness = evaluator.score(&baseline);
        let weighted_fitness = evaluator.score(&weighted);

        assert!(weighted_fitness.primary > baseline_fitness.primary);
        let delta = weighted_fitness.primary - baseline_fitness.primary;
        assert!(
            (delta - 0.003_333_333_333_333_393).abs() < 1e-9,
            "fitness delta drifted from the committed value: {delta}"
        );
        assert_eq!(baseline_fitness.authority, "none");
        assert_eq!(weighted_fitness.authority, "none");
    }
}
