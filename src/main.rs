use std::error::Error;

use clap::{Parser, ValueEnum};
use jitter_map::{PlanConfig, PlanSummary, RetryPlan, Strategy, generate_plan};

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Map retry storms before they happen",
    long_about = "Generate deterministic retry schedules for a client fleet and measure how often clients land in the same collision window."
)]
struct Cli {
    /// Number of independent clients in the simulated fleet.
    #[arg(long, default_value_t = 100)]
    clients: u32,

    /// Number of retry attempts scheduled per client.
    #[arg(long, default_value_t = 5)]
    attempts: u32,

    /// Initial retry delay (integer with ms, s, m, or h suffix).
    #[arg(long, default_value = "100ms", value_parser = parse_duration_ms)]
    base: u64,

    /// Maximum retry delay (integer with ms, s, m, or h suffix).
    #[arg(long, default_value = "30s", value_parser = parse_duration_ms)]
    cap: u64,

    /// Jitter algorithm to simulate.
    #[arg(long, value_enum, default_value_t = StrategyArg::Full)]
    strategy: StrategyArg,

    /// Seed that makes jittered schedules exactly reproducible.
    #[arg(long, default_value_t = 1)]
    seed: u64,

    /// Width used to group simultaneous retries (integer duration).
    #[arg(long, default_value = "10ms", value_parser = parse_duration_ms)]
    collision_window: u64,

    /// Output encoding.
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    format: OutputFormat,

    /// Emit aggregate metrics without individual retry events.
    #[arg(long)]
    summary_only: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum StrategyArg {
    Full,
    Equal,
    Decorrelated,
    None,
}

impl From<StrategyArg> for Strategy {
    fn from(value: StrategyArg) -> Self {
        match value {
            StrategyArg::Full => Self::Full,
            StrategyArg::Equal => Self::Equal,
            StrategyArg::Decorrelated => Self::Decorrelated,
            StrategyArg::None => Self::None,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Table,
    Json,
    Csv,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    let plan = generate_plan(PlanConfig {
        clients: cli.clients,
        attempts: cli.attempts,
        base_ms: cli.base,
        cap_ms: cli.cap,
        strategy: cli.strategy.into(),
        seed: cli.seed,
        collision_window_ms: cli.collision_window,
    })?;

    match (cli.format, cli.summary_only) {
        (OutputFormat::Json, false) => println!("{}", serde_json::to_string_pretty(&plan)?),
        (OutputFormat::Json, true) => {
            println!("{}", serde_json::to_string_pretty(&plan.summary)?)
        }
        (OutputFormat::Csv, false) => print_csv_events(&plan),
        (OutputFormat::Csv, true) => print_csv_summary(&plan.summary),
        (OutputFormat::Table, false) => print_table(&plan),
        (OutputFormat::Table, true) => print_table_summary(&plan.summary),
    }

    Ok(())
}

fn parse_duration_ms(input: &str) -> Result<u64, String> {
    let (digits, multiplier) = if let Some(value) = input.strip_suffix("ms") {
        (value, 1)
    } else if let Some(value) = input.strip_suffix('s') {
        (value, 1_000)
    } else if let Some(value) = input.strip_suffix('m') {
        (value, 60_000)
    } else if let Some(value) = input.strip_suffix('h') {
        (value, 3_600_000)
    } else {
        (input, 1)
    };

    let value = digits.parse::<u64>().map_err(|_| {
        format!("invalid duration '{input}': expected an integer with ms, s, m, or h")
    })?;
    value
        .checked_mul(multiplier)
        .ok_or_else(|| format!("duration '{input}' is too large"))
}

fn print_table(plan: &RetryPlan) {
    println!("client\tattempt\tdelay_ms\twake_at_ms");
    for event in &plan.events {
        println!(
            "{}\t{}\t{}\t{}",
            event.client, event.attempt, event.delay_ms, event.wake_at_ms
        );
    }
    println!();
    print_table_summary(&plan.summary);
}

fn print_table_summary(summary: &PlanSummary) {
    println!("total events: {}", summary.total_events);
    println!("occupied windows: {}", summary.occupied_windows);
    println!(
        "colliding client-windows: {}",
        summary.colliding_client_windows
    );
    println!("collision pairs: {}", summary.collision_pairs);
    println!("peak window load: {}", summary.peak_window_load);
}

fn print_csv_events(plan: &RetryPlan) {
    println!("client,attempt,delay_ms,wake_at_ms");
    for event in &plan.events {
        println!(
            "{},{},{},{}",
            event.client, event.attempt, event.delay_ms, event.wake_at_ms
        );
    }
}

fn print_csv_summary(summary: &PlanSummary) {
    println!("metric,value");
    println!("total_events,{}", summary.total_events);
    println!("occupied_windows,{}", summary.occupied_windows);
    println!(
        "colliding_client_windows,{}",
        summary.colliding_client_windows
    );
    println!("collision_pairs,{}", summary.collision_pairs);
    println!("peak_window_load,{}", summary.peak_window_load);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_supported_duration_units() {
        assert_eq!(parse_duration_ms("25ms"), Ok(25));
        assert_eq!(parse_duration_ms("3s"), Ok(3_000));
        assert_eq!(parse_duration_ms("2m"), Ok(120_000));
        assert_eq!(parse_duration_ms("1h"), Ok(3_600_000));
        assert_eq!(parse_duration_ms("90"), Ok(90));
    }

    #[test]
    fn rejects_fractional_and_overflowing_durations() {
        assert!(parse_duration_ms("1.5s").is_err());
        assert!(parse_duration_ms("18446744073709551615h").is_err());
    }
}
