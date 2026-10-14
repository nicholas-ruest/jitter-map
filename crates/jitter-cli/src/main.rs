//! Composition Root: `jitter-map`, the CLI JSON boundary.
//!
//! ADR-0120 (Composition Root — CLI; extends ADR-0024, ADR-0025);
//! docs/ddd/10-application-workflows.md. Preserves the base `3db7fd2` CLI's
//! stdin/stdout JSON contract and wires every real adapter into
//! `jitter-application`'s use case. The only crate in the workspace permitted to
//! depend on everything.

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use jitter_adapter_aws::AwsRetryConfigAdapter;
use jitter_adapter_ruflo::RufloSignalAdapter;
use jitter_adapter_ruvector::RuVectorMemoryAdapter;
use jitter_adapter_rvf::RvfWitnessAdapter;
use jitter_application::RunFleetRetryWorkload;
use jitter_outcome::AdmissionStrategy;
use jitter_scenario::Scenario;
use std::collections::HashMap;
use std::{fs, path::PathBuf};

#[derive(Parser)]
#[command(
    version,
    about = "Evaluate fleet retry admission without changing production authority"
)]
struct Cli {
    #[arg(long)]
    scenario: PathBuf,
    #[arg(long, value_enum, default_value_t = PolicyArg::Adaptive)]
    policy: PolicyArg,
    #[arg(long, default_value = ".jitter-map-memory")]
    memory: PathBuf,
    #[arg(long, default_value_t = 3)]
    aws_max_attempts: u32,
    #[arg(long, default_value_t = 5_000)]
    timeout_ms: u64,
    /// Simulate an orchestration-signaled interruption after this tick: capture a
    /// checkpoint and exit instead of running to completion (docs/architecture.md §7).
    #[arg(long)]
    interrupt_after: Option<u32>,
    /// Where to write the checkpoint captured by `--interrupt-after`.
    #[arg(long, default_value = ".jitter-map-checkpoint.json")]
    checkpoint_out: PathBuf,
    /// Resume a prior `--interrupt-after` run from a checkpoint file instead of
    /// running fresh.
    #[arg(long)]
    replay: Option<PathBuf>,
}

#[derive(Clone, Copy, ValueEnum)]
enum PolicyArg {
    Local,
    Aws,
    Adaptive,
    AdaptiveDeferral,
    TenantFairBudget,
    WeightedTenantFairBudget,
}

impl From<PolicyArg> for AdmissionStrategy {
    fn from(value: PolicyArg) -> Self {
        match value {
            PolicyArg::Local => AdmissionStrategy::LocalJitter,
            PolicyArg::Aws => AdmissionStrategy::AwsStandard,
            PolicyArg::Adaptive => AdmissionStrategy::AdaptiveBudget,
            PolicyArg::AdaptiveDeferral => AdmissionStrategy::AdaptiveDeferral,
            PolicyArg::TenantFairBudget => AdmissionStrategy::TenantFairBudget,
            PolicyArg::WeightedTenantFairBudget => AdmissionStrategy::WeightedTenantFairBudget,
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    let mut env: HashMap<String, String> = HashMap::new();
    env.insert(
        "JITTER_MEMORY_PATH".into(),
        cli.memory.to_string_lossy().into_owned(),
    );
    env.insert(
        "JITTER_AWS_MAX_ATTEMPTS".into(),
        cli.aws_max_attempts.to_string(),
    );
    env.insert("JITTER_TIMEOUT_MS".into(), cli.timeout_ms.to_string());
    let config = jitter_config::load_from(&env).map_err(anyhow::Error::msg)?;
    let _tracing_guard = jitter_observability::init_tracing(&config);

    let input = fs::read_to_string(&cli.scenario)
        .with_context(|| format!("read {}", cli.scenario.display()))?;
    let scenario: Scenario = serde_json::from_str(&input).context("parse scenario JSON")?;
    let seed = scenario.seed;
    let validated = jitter_scenario::validate(scenario).map_err(anyhow::Error::msg)?;
    let strategy: AdmissionStrategy = cli.policy.into();

    let retry_config = AwsRetryConfigAdapter::standard(config.aws_max_attempts);
    let memory =
        RuVectorMemoryAdapter::open(&config.memory_path).map_err(|e| anyhow::anyhow!(e))?;
    let workload =
        RunFleetRetryWorkload::new(retry_config, RvfWitnessAdapter, memory, strategy, seed);

    if let Some(checkpoint_path) = &cli.replay {
        let raw = fs::read_to_string(checkpoint_path)
            .with_context(|| format!("read checkpoint {}", checkpoint_path.display()))?;
        let checkpoint: jitter_recovery::Checkpoint =
            serde_json::from_str(&raw).context("parse checkpoint JSON")?;
        let remaining = jitter_workload::generate(&validated, seed);
        let run = jitter_recovery::replay(&checkpoint, remaining);
        let receipt = workload
            .finish_run(&validated, run)
            .map_err(|e| anyhow::anyhow!(e))?;
        println!("{}", serde_json::to_string_pretty(&receipt.receipt)?);
        return Ok(());
    }

    if let Some(interrupt_after) = cli.interrupt_after {
        let admission_config = workload.resolve_config().map_err(|e| anyhow::anyhow!(e))?;
        let checkpoint =
            jitter_recovery::capture(&validated, seed, &admission_config, interrupt_after);
        fs::write(
            &cli.checkpoint_out,
            serde_json::to_string_pretty(&checkpoint)?,
        )
        .with_context(|| format!("write checkpoint {}", cli.checkpoint_out.display()))?;
        println!(
            "{}",
            serde_json::json!({
                "interrupted_after_tick": interrupt_after,
                "checkpoint_path": cli.checkpoint_out,
            })
        );
        return Ok(());
    }

    let harness = RufloSignalAdapter::new();
    let receipt = workload
        .execute_with_recovery(validated, &harness)
        .map_err(|e| anyhow::anyhow!(e))?;
    println!("{}", serde_json::to_string_pretty(&receipt.receipt)?);
    Ok(())
}
