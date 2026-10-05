use jitter_domain::{PolicyKind, RetryPolicy, RunControl, simulate};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{self, Read},
};

struct Control;

impl RunControl for Control {
    fn cancelled(&self) -> bool {
        false
    }

    fn deadline_exceeded(&self) -> bool {
        false
    }
}

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
    scenario_sha256: String,
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

    let policy = RetryPolicy {
        kind: PolicyKind::AdaptiveBudget,
        max_attempts: 6,
        base_delay: 1,
        cap_delay: 32,
        shared_budget,
        refill_per_tick,
        failure_costs: BTreeMap::from([("throttling".into(), throttling_cost)]),
        upstream_revision: "metaharness-candidate".into(),
    };
    let scenario = jitter_evaluation::frozen_scenario();
    let report = simulate(&scenario, &policy, &Control).map_err(|error| error.to_string())?;

    let success_rate = f64::from(report.successes) / f64::from(report.original_requests);
    let raf = report.retry_amplification_factor();
    let score = ScoreCard {
        primary: success_rate - 0.02 * raf,
        regressed: success_rate < 0.9,
        noop_rate: f64::from(report.terminal_failures) / f64::from(report.original_requests),
        cost_per_win: report.attempts as f64 / f64::from(report.successes.max(1)),
        raw: RawEvidence {
            variant_id: candidate.variant_id,
            retry_amplification_factor: raf,
            successes: report.successes,
            attempts: report.attempts,
            denied_retries: report.denied_retries,
            p95_ticks: report.p95_ticks(),
            scenario_sha256: report.scenario_sha256,
            authority: report.authority,
        },
    };
    println!(
        "{}",
        serde_json::to_string(&score).map_err(|error| error.to_string())?
    );
    Ok(())
}
