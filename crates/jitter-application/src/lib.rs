use jitter_domain::{PolicyKind, RetryPolicy, RunControl, Scenario, SimulationReport, simulate};
use serde::Serialize;
use std::time::{Duration, Instant};
use thiserror::Error;
pub trait PolicySource {
    fn policy(&self, kind: PolicyKind) -> Result<RetryPolicy, String>;
}
pub trait OutcomeMemory {
    fn append_and_search(&self, report: &SimulationReport) -> Result<Vec<String>, String>;
}
pub trait WitnessSealer {
    fn seal(&self, actions: &[Vec<u8>]) -> Result<String, String>;
}
pub trait Clock {
    fn now(&self) -> Instant;
}
pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}
#[derive(Debug)]
pub struct EvaluateRequest {
    pub scenario: Scenario,
    pub kind: PolicyKind,
    pub timeout_ms: u64,
}
#[derive(Debug, Serialize)]
pub struct EvaluationReceipt {
    pub report: SimulationReport,
    pub retry_amplification_factor: f64,
    pub p95_ticks: u32,
    pub nearest_prior_ids: Vec<String>,
    pub witness_root: String,
    pub authority: String,
}
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("policy: {0}")]
    Policy(String),
    #[error("simulation: {0}")]
    Domain(#[from] jitter_domain::DomainError),
    #[error("witness: {0}")]
    Witness(String),
    #[error("memory: {0}")]
    Memory(String),
}
struct Control {
    started: Instant,
    limit: Duration,
}
impl RunControl for Control {
    fn cancelled(&self) -> bool {
        false
    }
    fn deadline_exceeded(&self) -> bool {
        self.started.elapsed() > self.limit
    }
}
pub fn evaluate(
    req: EvaluateRequest,
    policies: &impl PolicySource,
    memory: &impl OutcomeMemory,
    sealer: &impl WitnessSealer,
    clock: &impl Clock,
) -> Result<EvaluationReceipt, ApplicationError> {
    let policy = policies
        .policy(req.kind)
        .map_err(ApplicationError::Policy)?;
    let report = simulate(
        &req.scenario,
        &policy,
        &Control {
            started: clock.now(),
            limit: Duration::from_millis(req.timeout_ms),
        },
    )?;
    let actions = vec![
        report.scenario_sha256.as_bytes().to_vec(),
        format!("{policy:?}").into_bytes(),
        format!("{report:?}").into_bytes(),
    ];
    let witness_root = sealer.seal(&actions).map_err(ApplicationError::Witness)?;
    let nearest_prior_ids = memory
        .append_and_search(&report)
        .map_err(ApplicationError::Memory)?;
    Ok(EvaluationReceipt {
        retry_amplification_factor: report.retry_amplification_factor(),
        p95_ticks: report.p95_ticks(),
        report,
        nearest_prior_ids,
        witness_root,
        authority: "none".into(),
    })
}
