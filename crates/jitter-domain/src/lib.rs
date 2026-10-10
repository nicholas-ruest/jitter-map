use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cmp::Reverse;
use std::collections::{BTreeMap, BinaryHeap, VecDeque};
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
    AdaptiveDeferral,
    TenantFairBudget,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tenant_weights: Option<Vec<u32>>,
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
    pub tenant_original_requests: Vec<u32>,
    pub tenant_attempts: Vec<u64>,
    pub tenant_successes: Vec<u32>,
    pub tenant_terminal_failures: Vec<u32>,
    pub tenant_denied_retries: Vec<u64>,
    pub tenant_deferred_retries: Vec<u64>,
    pub deferred_retries: u64,
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
    pub fn tenant_success_fairness(&self) -> Option<f64> {
        let ratios: Vec<f64> = self
            .tenant_original_requests
            .iter()
            .zip(&self.tenant_successes)
            .filter_map(|(&requests, &successes)| {
                (requests > 0).then_some(f64::from(successes) / f64::from(requests))
            })
            .collect();
        let sum: f64 = ratios.iter().sum();
        let sum_squares: f64 = ratios.iter().map(|value| value * value).sum();
        if ratios.is_empty() || sum_squares == 0.0 {
            None
        } else {
            Some(sum * sum / (ratios.len() as f64 * sum_squares))
        }
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
    #[error("tenant weights must match tenant count, be positive, and have a safe sum")]
    TenantWeights,
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
    if let Some(weights) = &s.tenant_weights {
        if weights.len() != s.tenants as usize
            || weights.contains(&0)
            || weights
                .iter()
                .try_fold(0u32, |sum, weight| sum.checked_add(*weight))
                .is_none()
        {
            return Err(DomainError::TenantWeights);
        }
    }
    Ok(())
}
type Event = (u32, u32, u32, u32);

fn tenant_for_request(s: &Scenario, id: u32) -> u32 {
    let Some(weights) = &s.tenant_weights else {
        return id % s.tenants;
    };
    let cycle: u32 = weights.iter().sum();
    let mut offset = id % cycle;
    for (tenant, weight) in weights.iter().enumerate() {
        if offset < *weight {
            return tenant as u32;
        }
        offset -= *weight;
    }
    unreachable!("validated tenant weights cover the cycle")
}

fn fair_order(batch: Vec<Event>, tick: u32, tenants: u32) -> Vec<Event> {
    let mut queues = vec![VecDeque::new(); tenants as usize];
    for event in batch {
        queues[event.1 as usize].push_back(event);
    }
    let start = tick % tenants;
    let mut ordered = Vec::new();
    loop {
        let before = ordered.len();
        for offset in 0..tenants {
            let tenant = (start + offset) % tenants;
            if let Some(event) = queues[tenant as usize].pop_front() {
                ordered.push(event);
            }
        }
        if ordered.len() == before {
            break;
        }
    }
    ordered
}

pub fn simulate(
    s: &Scenario,
    p: &RetryPolicy,
    ctl: &impl RunControl,
) -> Result<SimulationReport, DomainError> {
    validate(s)?;
    if p.max_attempts == 0 || p.base_delay == 0 || p.base_delay > p.cap_delay {
        return Err(DomainError::Policy);
    }
    let mut q: BinaryHeap<Reverse<Event>> = BinaryHeap::new();
    let mut tenant_original_requests = vec![0u32; s.tenants as usize];
    for id in 0..s.requests {
        let tenant = tenant_for_request(s, id);
        tenant_original_requests[tenant as usize] += 1;
        q.push(Reverse((0, tenant, id, 1)))
    }
    let (mut attempts, mut successes, mut terminal, mut denied, mut deferred) =
        (0u64, 0u32, 0u32, 0u64, 0u64);
    let mut tenant_attempts = vec![0u64; s.tenants as usize];
    let mut tenant_successes = vec![0u32; s.tenants as usize];
    let mut tenant_terminal_failures = vec![0u32; s.tenants as usize];
    let mut tenant_denied_retries = vec![0u64; s.tenants as usize];
    let mut tenant_deferred_retries = vec![0u64; s.tenants as usize];
    let mut completed = Vec::new();
    let mut budget = p.shared_budget;
    let mut last = 0;
    let mut used = BTreeMap::<u32, u32>::new();
    while let Some(Reverse(first)) = q.pop() {
        let tick = first.0;
        let mut batch = vec![first];
        while q.peek().is_some_and(|Reverse(event)| event.0 == tick) {
            batch.push(q.pop().expect("peeked event exists").0);
        }
        let batch = if p.kind == PolicyKind::TenantFairBudget {
            fair_order(batch, tick, s.tenants)
        } else {
            batch
        };
        for (tick, tenant, id, attempt) in batch {
            if ctl.cancelled() {
                return Err(DomainError::Cancelled);
            }
            if ctl.deadline_exceeded() {
                return Err(DomainError::Deadline);
            }
            if attempts >= s.max_events {
                return Err(DomainError::EventLimit);
            }
            if tick > s.horizon_ticks {
                terminal += 1;
                tenant_terminal_failures[tenant as usize] += 1;
                continue;
            }
            if tick > last {
                budget = budget
                    .saturating_add((tick - last).saturating_mul(p.refill_per_tick))
                    .min(p.shared_budget);
                last = tick
            }
            attempts += 1;
            tenant_attempts[tenant as usize] += 1;
            let slot = used.entry(tick).or_default();
            if tick >= s.outage_ticks && *slot < s.capacity_per_tick {
                *slot += 1;
                successes += 1;
                tenant_successes[tenant as usize] += 1;
                completed.push(tick);
                continue;
            }
            if attempt >= p.max_attempts || s.failure == FailureClass::Permanent {
                terminal += 1;
                tenant_terminal_failures[tenant as usize] += 1;
                continue;
            }
            let cost = *p.failure_costs.get(failure_name(s.failure)).unwrap_or(&1);
            let budget_gated = matches!(
                p.kind,
                PolicyKind::AdaptiveBudget
                    | PolicyKind::AdaptiveDeferral
                    | PolicyKind::TenantFairBudget
            );
            if budget_gated && budget < cost {
                if p.kind == PolicyKind::AdaptiveDeferral && p.refill_per_tick > 0 {
                    let wait = (cost - budget).div_ceil(p.refill_per_tick);
                    let deferred_tick = tick.saturating_add(wait);
                    if deferred_tick <= s.horizon_ticks {
                        deferred += 1;
                        tenant_deferred_retries[tenant as usize] += 1;
                        budget = 0;
                        q.push(Reverse((deferred_tick, tenant, id, attempt + 1)));
                        continue;
                    }
                }
                denied += 1;
                tenant_denied_retries[tenant as usize] += 1;
                terminal += 1;
                tenant_terminal_failures[tenant as usize] += 1;
                continue;
            }
            if budget_gated {
                budget -= cost
            }
            let ceiling = p
                .base_delay
                .saturating_mul(
                    1u32.checked_shl(attempt.saturating_sub(1))
                        .unwrap_or(u32::MAX),
                )
                .min(p.cap_delay);
            let delay = 1 + (random_word(s.seed, id, attempt) % u64::from(ceiling)) as u32;
            q.push(Reverse((
                tick.saturating_add(delay),
                tenant,
                id,
                attempt + 1,
            )))
        }
    }
    Ok(SimulationReport {
        policy: p.kind,
        original_requests: s.requests,
        attempts,
        successes,
        terminal_failures: terminal,
        denied_retries: denied,
        tenant_original_requests,
        tenant_attempts,
        tenant_successes,
        tenant_terminal_failures,
        tenant_denied_retries,
        tenant_deferred_retries,
        deferred_retries: deferred,
        completed_at: completed,
        scenario_sha256: scenario_digest(s),
        authority: "none".into(),
    })
}
pub fn scenario_digest(s: &Scenario) -> String {
    let mut h = Sha256::new();
    h.update(format!(
        "{}:{}:{}:{}:{}:{}:{}:{:?}:{:?}:{}",
        s.version,
        s.requests,
        s.tenants,
        s.capacity_per_tick,
        s.outage_ticks,
        s.horizon_ticks,
        s.seed,
        s.failure,
        s.tenant_weights,
        s.max_events
    ));
    format!("{:x}", h.finalize())
}
fn failure_name(f: FailureClass) -> &'static str {
    match f {
        FailureClass::Transient => "transient",
        FailureClass::Throttling => "throttling",
        FailureClass::Timeout => "timeout",
        FailureClass::Permanent => "permanent",
    }
}
fn random_word(seed: u64, id: u32, attempt: u32) -> u64 {
    let mut x = seed ^ u64::from(id).wrapping_mul(0x9e3779b97f4a7c15) ^ u64::from(attempt);
    x = x.wrapping_add(0x9e3779b97f4a7c15);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^ (x >> 31)
}
fn quantile(values: &[u32], pct: usize) -> u32 {
    if values.is_empty() {
        return 0;
    }
    let mut v = values.to_vec();
    v.sort_unstable();
    v[((v.len() - 1) * pct / 100).min(v.len() - 1)]
}
#[cfg(test)]
mod tests {
    use super::*;
    struct C;
    impl RunControl for C {
        fn cancelled(&self) -> bool {
            false
        }
        fn deadline_exceeded(&self) -> bool {
            false
        }
    }
    fn s() -> Scenario {
        Scenario {
            version: 1,
            requests: 100,
            tenants: 4,
            capacity_per_tick: 10,
            outage_ticks: 3,
            horizon_ticks: 100,
            seed: 7,
            failure: FailureClass::Throttling,
            tenant_weights: None,
            max_events: 10000,
        }
    }
    fn p(k: PolicyKind) -> RetryPolicy {
        RetryPolicy {
            kind: k,
            max_attempts: 5,
            base_delay: 1,
            cap_delay: 16,
            shared_budget: 40,
            refill_per_tick: 2,
            failure_costs: BTreeMap::from([("throttling".into(), 3)]),
            upstream_revision: "test".into(),
        }
    }
    #[test]
    fn replay_is_stable() {
        assert_eq!(
            simulate(&s(), &p(PolicyKind::AdaptiveBudget), &C).unwrap(),
            simulate(&s(), &p(PolicyKind::AdaptiveBudget), &C).unwrap()
        )
    }
    #[test]
    fn budget_limits_amplification() {
        let a = simulate(&s(), &p(PolicyKind::AdaptiveBudget), &C).unwrap();
        let l = simulate(&s(), &p(PolicyKind::LocalJitter), &C).unwrap();
        assert!(a.attempts < l.attempts);
        assert!(a.denied_retries > 0)
    }
    #[test]
    fn invalid_is_rejected() {
        let mut x = s();
        x.requests = 0;
        assert_eq!(validate(&x), Err(DomainError::Zero))
    }
    #[test]
    fn deferral_recovers_denied_budget() {
        let budget = simulate(&s(), &p(PolicyKind::AdaptiveBudget), &C).unwrap();
        let deferral = simulate(&s(), &p(PolicyKind::AdaptiveDeferral), &C).unwrap();
        assert!(deferral.deferred_retries > 0);
        assert!(deferral.successes > budget.successes);
        assert!(deferral.denied_retries <= budget.denied_retries);
    }
    #[test]
    fn deferral_fails_closed_without_refill() {
        let mut budget_policy = p(PolicyKind::AdaptiveBudget);
        budget_policy.refill_per_tick = 0;
        let mut deferral_policy = p(PolicyKind::AdaptiveDeferral);
        deferral_policy.refill_per_tick = 0;
        let budget = simulate(&s(), &budget_policy, &C).unwrap();
        let deferral = simulate(&s(), &deferral_policy, &C).unwrap();
        assert_eq!(deferral.deferred_retries, 0);
        assert_eq!(deferral.denied_retries, budget.denied_retries);
        assert_eq!(deferral.successes, budget.successes);
        assert_eq!(deferral.attempts, budget.attempts);
    }
    #[test]
    fn deferral_replay_is_stable() {
        assert_eq!(
            simulate(&s(), &p(PolicyKind::AdaptiveDeferral), &C).unwrap(),
            simulate(&s(), &p(PolicyKind::AdaptiveDeferral), &C).unwrap()
        )
    }
    #[test]
    fn deferral_fails_closed_beyond_horizon() {
        let scenario = Scenario {
            version: 1,
            requests: 10,
            tenants: 2,
            capacity_per_tick: 5,
            outage_ticks: 2,
            horizon_ticks: 3,
            seed: 1,
            failure: FailureClass::Throttling,
            tenant_weights: None,
            max_events: 1_000,
        };
        let policy = RetryPolicy {
            kind: PolicyKind::AdaptiveDeferral,
            max_attempts: 5,
            base_delay: 1,
            cap_delay: 4,
            shared_budget: 1,
            refill_per_tick: 1,
            failure_costs: BTreeMap::from([("throttling".into(), 100)]),
            upstream_revision: "test".into(),
        };
        let report = simulate(&scenario, &policy, &C).unwrap();
        assert_eq!(report.deferred_retries, 0);
        assert!(report.denied_retries > 0);
    }
}
