use jitter_outcome::AdmissionStrategy;
use jitter_simulation::AdmissionConfig;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{self, Read},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Candidate {
    variant_id: String,
    genome: BTreeMap<String, f64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScoreCard {
    primary: f64,
    regressed: bool,
    noop_rate: f64,
    cost_per_win: f64,
    raw: RawEvidence,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RawEvidence {
    variant_id: String,
    retry_amplification_factor: f64,
    successes: u32,
    attempts: u64,
    denied_retries: u64,
    p95_ticks: u32,
    scenario_digest: String,
    authority: String,
}

fn required(genome: &BTreeMap<String, f64>, key: &str) -> Result<u32, String> {
    let value = genome
        .get(key)
        .copied()
        .ok_or_else(|| format!("missing genome parameter {key}"))?;
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > f64::from(u32::MAX) {
        return Err(format!("invalid integer genome parameter {key}"));
    }
    Ok(value as u32)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| error.to_string())?;
    let candidate: Candidate = serde_json::from_str(&input).map_err(|error| error.to_string())?;
    let shared_budget = required(&candidate.genome, "shared_budget")?;
    let refill_per_tick = required(&candidate.genome, "refill_per_tick")?;
    let throttling_cost = required(&candidate.genome, "throttling_cost")?;
    if shared_budget == 0 || refill_per_tick == 0 || throttling_cost == 0 {
        return Err("retry budget parameters must be positive".into());
    }

    let config = AdmissionConfig {
        strategy: AdmissionStrategy::AdaptiveBudget,
        max_attempts: 6,
        base_delay: 1,
        cap_delay: 32,
        shared_budget,
        refill_per_tick,
        failure_cost: throttling_cost,
    };
    let scenario = jitter_evaluation::frozen_scenario();
    let run = jitter_simulation::run(&scenario, scenario.scenario().seed, &config);

    let success_rate = f64::from(run.summary.successes) / f64::from(run.original_requests);
    let raf = run.retry_amplification_factor();
    let score = ScoreCard {
        primary: success_rate - 0.02 * raf,
        regressed: success_rate < 0.9,
        noop_rate: f64::from(run.summary.terminal_failures) / f64::from(run.original_requests),
        cost_per_win: run.summary.attempts as f64 / f64::from(run.summary.successes.max(1)),
        raw: RawEvidence {
            variant_id: candidate.variant_id,
            retry_amplification_factor: raf,
            successes: run.summary.successes,
            attempts: run.summary.attempts,
            denied_retries: run.summary.denied_retries,
            p95_ticks: run.p95_ticks(),
            scenario_digest: run.scenario_digest,
            authority: "none".into(),
        },
    };
    println!(
        "{}",
        serde_json::to_string(&score).map_err(|error| error.to_string())?
    );
    Ok(())
}
