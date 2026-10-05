use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap};
use thiserror::Error;
pub const MAX_EVENTS: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    Transient,
    Throttling,
    Timeout,
    Permanent,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyKind {
    LocalJitter,
    AwsStandard,
    AdaptiveBudget,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub version: u32,
    pub requests: u32,
    pub tenants: u32,
    pub capacity_per_tick: u32,
    pub outage_ticks: u32,
    pub horizon_ticks: u32,
    pub seed: u64,
    pub failure: FailureClass,
    pub max_events: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub kind: PolicyKind,
    pub max_attempts: u32,
    pub base_delay: u32,
    pub cap_delay: u32,
    pub shared_budget: u32,
    pub refill_per_tick: u32,
    pub failure_costs: BTreeMap<String, u32>,
    pub upstream_revision: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulationReport {
    pub policy: PolicyKind,
    pub original_requests: u32,
    pub attempts: u64,
    pub successes: u32,
    pub terminal_failures: u32,
    pub denied_retries: u64,
    pub completed_at: Vec<u32>,
    pub scenario_sha256: String,
    pub authority: String,
}
impl SimulationReport {
    pub fn retry_amplification_factor(&self) -> f64 {
        self.attempts as f64 / f64::from(self.original_requests)
    }
    pub fn p95_ticks(&self) -> u32 {
        quantile(&self.completed_at, 95)
    }
}
#[derive(Debug, Error, PartialEq, Eq)]
pub enum DomainError {
    #[error("scenario version must be 1")]
    Version,
    #[error("scenario counts and capacity must be positive")]
    Zero,
    #[error("scenario horizon must exceed outage duration")]
    Horizon,
    #[error("maximum events must be between request count and {MAX_EVENTS}")]
    EventLimit,
    #[error("policy delays and attempts must be valid")]
    Policy,
    #[error("simulation cancelled")]
    Cancelled,
    #[error("simulation deadline exceeded")]
    Deadline,
}
pub trait RunControl {
    fn cancelled(&self) -> bool;
    fn deadline_exceeded(&self) -> bool;
}
pub fn validate(s: &Scenario) -> Result<(), DomainError> {
    if s.version != 1 {
        return Err(DomainError::Version);
    }
    if s.requests == 0 || s.tenants == 0 || s.capacity_per_tick == 0 {
        return Err(DomainError::Zero);
    }
    if s.horizon_ticks <= s.outage_ticks {
        return Err(DomainError::Horizon);
    }
    if s.max_events < u64::from(s.requests) || s.max_events > MAX_EVENTS {
        return Err(DomainError::EventLimit);
    }
    Ok(())
}
pub fn simulate(
    s: &Scenario,
    p: &RetryPolicy,
    ctl: &impl RunControl,