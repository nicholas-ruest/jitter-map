//! Composition Root: `jitter-service`, a long-running executable exposing a `/run`
//! endpoint over a minimal HTTP boundary, plus `/evaluate` routing through
//! `jitter-evaluation` and `jitter-adapter-metaharness`.
//!
//! ADR-0119 (Composition Root — Service); docs/ddd/10-application-workflows.md.
//! Permitted to depend on everything — the other composition root alongside
//! `jitter-cli`.
//!
//! The HTTP layer is deliberately minimal (hand-rolled over `std::net::TcpListener`,
//! not a web framework): this crate's job is to prove the composition wiring, not to
//! be a production HTTP server.

use jitter_adapter_aws::AwsRetryConfigAdapter;
use jitter_adapter_metaharness::MetaharnessAdapter;
use jitter_adapter_ruvector::RuVectorMemoryAdapter;
use jitter_adapter_rvf::RvfWitnessAdapter;
use jitter_application::RunFleetRetryWorkload;
use jitter_config::AppConfig;
use jitter_outcome::AdmissionStrategy;
use jitter_ports::{CandidateResult, FlywheelSink};
use jitter_scenario::Scenario;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

pub type App =
    RunFleetRetryWorkload<AwsRetryConfigAdapter, RvfWitnessAdapter, RuVectorMemoryAdapter>;

/// Wires every adapter this service uses into one `RunFleetRetryWorkload`, exactly
/// as the planning table's `build_app(AppConfig) -> RunFleetRetryWorkload<..>` entry
/// names it.
pub fn build_app(
    config: &AppConfig,
    strategy: AdmissionStrategy,
    seed: u64,
) -> std::io::Result<App> {
    let memory = RuVectorMemoryAdapter::open(&config.memory_path)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(RunFleetRetryWorkload::new(
        AwsRetryConfigAdapter::standard(config.aws_max_attempts),
        RvfWitnessAdapter,
        memory,
        strategy,
        seed,
    ))
}

#[derive(Deserialize)]
struct RunRequest {
    scenario: Scenario,
    #[serde(default)]
    strategy: Option<AdmissionStrategy>,
}

#[derive(Deserialize)]
struct EvaluateRequest {
    candidate_id: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

fn default_strategy() -> AdmissionStrategy {
    AdmissionStrategy::WeightedTenantFairBudget
}

fn handle_run(config: &AppConfig, body: &str) -> (u16, String) {
    let request: RunRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => return (400, json_error(format!("invalid request body: {e}"))),
    };
    let strategy = request.strategy.unwrap_or_else(default_strategy);
    let seed = request.scenario.seed;
    let validated = match jitter_scenario::validate(request.scenario) {
        Ok(v) => v,
        Err(e) => return (400, json_error(e.to_string())),
    };
    let app = match build_app(config, strategy, seed) {
        Ok(a) => a,
        Err(e) => return (503, json_error(e.to_string())),
    };
    match app.execute(validated) {
        Ok(receipt) => (
            200,
            serde_json::to_string(&receipt.receipt).unwrap_or_default(),
        ),
        Err(e) => (502, json_error(e.to_string())),
    }
}

fn handle_evaluate(config: &AppConfig, body: &str) -> (u16, String) {
    let request: EvaluateRequest = match serde_json::from_str(body) {
        Ok(r) => r,
        Err(e) => return (400, json_error(format!("invalid request body: {e}"))),
    };
    let scenario = jitter_evaluation::fairness_scenario();
    let baseline = jitter_simulation::run(
        &scenario,
        scenario.scenario().seed,
        &jitter_simulation::AdmissionConfig {
            strategy: AdmissionStrategy::TenantFairBudget,
            max_attempts: 6,
            base_delay: 1,
            cap_delay: 32,
            shared_budget: 1500,
            refill_per_tick: 18,
            failure_cost: 1,
        },
    );
    let weighted = jitter_simulation::run(
        &scenario,
        scenario.scenario().seed,
        &jitter_simulation::AdmissionConfig {
            strategy: AdmissionStrategy::WeightedTenantFairBudget,
            max_attempts: 6,
            base_delay: 1,
            cap_delay: 32,
            shared_budget: 1500,
            refill_per_tick: 18,
            failure_cost: 1,
        },
    );
    let evaluator = jitter_evaluation::SuccessRateMinusAmplification;
    use jitter_evaluation::CandidateEvaluator;
    let fitness = evaluator.score(&weighted);
    let baseline_fitness = evaluator.score(&baseline);

    let result = CandidateResult {
        candidate_id: request.candidate_id,
        fitness: fitness.primary,
        scenario_digest: weighted.scenario_digest.clone(),
    };
    let sink = MetaharnessAdapter::new(&config.flywheel_feed_path);
    if let Err(e) = sink.submit(&result) {
        return (502, json_error(e.to_string()));
    }
    (
        200,
        serde_json::to_string(&serde_json::json!({
            "weighted_fitness": fitness.primary,
            "baseline_fitness": baseline_fitness.primary,
            "outranks_baseline": fitness.primary > baseline_fitness.primary,
            "authority": "none",
        }))
        .unwrap_or_default(),
    )
}

fn json_error(message: String) -> String {
    serde_json::to_string(&ErrorBody { error: message }).unwrap_or_default()
}

struct ParsedRequest {
    method: String,
    path: String,
    body: String,
}

fn read_request(stream: &mut TcpStream) -> std::io::Result<ParsedRequest> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default().to_string();
    let path = parts.next().unwrap_or_default().to_string();

    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
            content_length = value.trim().parse().unwrap_or(0);
        }
    }
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;
    Ok(ParsedRequest {
        method,
        path,
        body: String::from_utf8_lossy(&body).into_owned(),
    })
}

fn write_response(stream: &mut TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    let status_text = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        502 => "Bad Gateway",
        _ => "Service Unavailable",
    };
    write!(
        stream,
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    )
}

/// Serves exactly `max_requests` requests on `listener`, then returns. Production
/// use calls this with `usize::MAX` from `main`; tests call it with a small bound so
/// the black-box integration test terminates.
pub fn serve(
    listener: &TcpListener,
    config: &AppConfig,
    max_requests: usize,
) -> std::io::Result<()> {
    for stream in listener.incoming().take(max_requests) {
        let mut stream = stream?;
        let request = read_request(&mut stream)?;
        let (status, body) = match (request.method.as_str(), request.path.as_str()) {
            ("POST", "/run") => handle_run(config, &request.body),
            ("POST", "/evaluate") => handle_evaluate(config, &request.body),
            _ => (404, json_error("not found".into())),
        };
        write_response(&mut stream, status, &body)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jitter_scenario::FailureClass;
    use std::net::TcpStream;
    use tempfile::tempdir;

    fn test_config(dir: &std::path::Path) -> AppConfig {
        AppConfig {
            memory_path: dir.join("memory"),
            aws_max_attempts: 3,
            timeout_ms: 5_000,
            flywheel_feed_path: dir.join("flywheel.jsonl"),
        }
    }

    fn post(addr: std::net::SocketAddr, path: &str, body: &str) -> (u16, String) {
        let mut stream = TcpStream::connect(addr).unwrap();
        write!(
            stream,
            "POST {path} HTTP/1.1\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        )
        .unwrap();
        stream.flush().unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        let status: u16 = response.split_whitespace().nth(1).unwrap().parse().unwrap();
        let body = response
            .split("\r\n\r\n")
            .nth(1)
            .unwrap_or_default()
            .to_string();
        (status, body)
    }

    /// Row 19's acceptance criterion: a black-box integration test posting
    /// `fleet-retry-storm-22`-shaped input to the running service.
    #[test]
    fn black_box_run_endpoint_returns_a_receipt() {
        let dir = tempdir().unwrap();
        let config = test_config(dir.path());
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || serve(&listener, &config, 1));

        let scenario = Scenario {
            version: 1,
            requests: 50,
            tenants: 5,
            capacity_per_tick: 10,
            outage_ticks: 2,
            horizon_ticks: 50,
            seed: 7,
            failure: FailureClass::Throttling,
            tenant_weights: None,
            max_events: 10_000,
        };
        let body = serde_json::to_string(&serde_json::json!({ "scenario": scenario })).unwrap();
        let (status, response_body) = post(addr, "/run", &body);
        handle.join().unwrap().unwrap();

        assert_eq!(status, 200);
        assert!(
            response_body.contains("\"authority\":\"none\""),
            "{response_body}"
        );
    }

    #[test]
    fn unknown_path_returns_404() {
        let dir = tempdir().unwrap();
        let config = test_config(dir.path());
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || serve(&listener, &config, 1));

        let (status, _) = post(addr, "/missing", "{}");
        handle.join().unwrap().unwrap();
        assert_eq!(status, 404);
    }

    #[test]
    fn evaluate_endpoint_submits_to_metaharness_and_reports_outranking() {
        let dir = tempdir().unwrap();
        let config = test_config(dir.path());
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || serve(&listener, &config, 1));

        let body =
            serde_json::to_string(&serde_json::json!({ "candidate_id": "candidate-a" })).unwrap();
        let (status, response_body) = post(addr, "/evaluate", &body);
        handle.join().unwrap().unwrap();

        assert_eq!(status, 200);
        assert!(
            response_body.contains("\"outranks_baseline\":true"),
            "{response_body}"
        );
    }
}
