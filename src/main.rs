use std::{fs, path::PathBuf};
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use jitter_adapter_aws::AwsPolicySource;
use jitter_adapter_ruvnet::{RuvectorMemory, RvfSealer};
use jitter_application::{evaluate, EvaluateRequest, SystemClock};
use jitter_domain::{PolicyKind, Scenario};

#[derive(Parser)]
#[command(version, about = "Evaluate fleet retry admission without changing production authority")]
struct Cli {
    #[arg(long)] scenario: PathBuf,
    #[arg(long, value_enum, default_value_t = PolicyArg::Adaptive)] policy: PolicyArg,
    #[arg(long, default_value = ".jitter-map-memory")] memory: PathBuf,
    #[arg(long, default_value_t = 3)] aws_max_attempts: u32,
    #[arg(long, default_value_t = 5_000)] timeout_ms: u64,
}
#[derive(Clone, Copy, ValueEnum)] enum PolicyArg { Local, Aws, Adaptive }
fn main() { if let Err(error) = run() { eprintln!("error: {error:#}"); std::process::exit(1); } }
fn run() -> Result<()> {
    let cli = Cli::parse();
    let input = fs::read_to_string(&cli.scenario).with_context(|| format!("read {}", cli.scenario.display()))?;
    let scenario: Scenario = serde_json::from_str(&input).context("parse scenario JSON")?;
    let kind = match cli.policy { PolicyArg::Local => PolicyKind::LocalJitter, PolicyArg::Aws => PolicyKind::AwsStandard, PolicyArg::Adaptive => PolicyKind::AdaptiveBudget };
    let memory = RuvectorMemory::open(&cli.memory).map_err(anyhow::Error::msg)?;
    let receipt = evaluate(EvaluateRequest { scenario, kind, timeout_ms: cli.timeout_ms }, &AwsPolicySource::standard(cli.aws_max_attempts), &memory, &RvfSealer, &SystemClock)?;
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}
