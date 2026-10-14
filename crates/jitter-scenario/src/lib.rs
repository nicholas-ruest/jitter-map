//! Domain Core: `Scenario` and `ValidatedScenario` value objects.
//!
//! ADR-0101 (Domain Purity Boundary); docs/ddd/03-scenario-model.md.
//! This crate has zero dependencies outside `serde`/`sha2`/`thiserror` — no other
//! workspace crate, domain or otherwise, is named here, by construction.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// Hard ceiling on simulated events, shared by validation and the orchestration layer.
pub const MAX_EVENTS: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    Transient,
    Throttling,
    Timeout,
    Permanent,
}

impl FailureClass {
    pub fn name(self) -> &'static str {
        match self {
            FailureClass::Transient => "transient",
            FailureClass::Throttling => "throttling",
            FailureClass::Timeout => "timeout",
            FailureClass::Permanent => "permanent",
        }
    }
}

/// Raw, untrusted scenario input, exactly the shape accepted at the CLI/service boundary.
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ScenarioError {
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
}

/// A `Scenario` that has passed every invariant in [`validate`]. Every invariant is
/// enforced exactly once, here, at construction — no downstream crate re-checks them
/// (ADR-0101).
#[derive(Debug, Clone)]
pub struct ValidatedScenario {
    inner: Scenario,
    digest: String,
}

impl ValidatedScenario {
    pub fn scenario(&self) -> &Scenario {
        &self.inner
    }

    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn tenants(&self) -> u32 {
        self.inner.tenants
    }

    pub fn requests(&self) -> u32 {
        self.inner.requests
    }

    pub fn tenant_weights(&self) -> Option<&[u32]> {
        self.inner.tenant_weights.as_deref()
    }

    /// Resolves the tenant a given request id belongs to, honoring demand weights when
    /// present. The assignment is a deterministic cyclic distribution over the weight
    /// sequence so that tenant load matches the configured demand profile exactly.
    pub fn tenant_for_request(&self, id: u32) -> u32 {
        let Some(weights) = &self.inner.tenant_weights else {
            return id % self.inner.tenants;
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
}

/// Validates a raw `Scenario`, rejecting invalid input before any allocation and never
/// panicking on malformed input.
pub fn validate(s: Scenario) -> Result<ValidatedScenario, ScenarioError> {
    if s.version != 1 {
        return Err(ScenarioError::Version);
    }
    if s.requests == 0 || s.tenants == 0 || s.capacity_per_tick == 0 {
        return Err(ScenarioError::Zero);
    }
    if s.horizon_ticks <= s.outage_ticks {
        return Err(ScenarioError::Horizon);
    }
    if s.max_events < u64::from(s.requests) || s.max_events > MAX_EVENTS {
        return Err(ScenarioError::EventLimit);
    }
    if let Some(weights) = &s.tenant_weights {
        if weights.len() != s.tenants as usize
            || weights.contains(&0)
            || weights
                .iter()
                .try_fold(0u32, |sum, weight| sum.checked_add(*weight))
                .is_none()
        {
            return Err(ScenarioError::TenantWeights);
        }
    }
    let digest = scenario_digest(&s);
    Ok(ValidatedScenario { inner: s, digest })
}

/// SHA-256 digest binding every field of a `Scenario`, including optional tenant
/// weights, so the digest changes whenever any input to the simulation changes
/// (ADR-0011 evidence-bound configuration).
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

#[cfg(test)]
mod tests {
    use super::*;

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
            max_events: 10_000,
        }
    }

    #[test]
    fn valid_scenario_is_accepted() {
        assert!(validate(s()).is_ok());
    }

    #[test]
    fn zero_requests_is_rejected() {
        let mut x = s();
        x.requests = 0;
        assert_eq!(validate(x).unwrap_err(), ScenarioError::Zero);
    }

    #[test]
    fn horizon_must_exceed_outage() {
        let mut x = s();
        x.horizon_ticks = x.outage_ticks;
        assert_eq!(validate(x).unwrap_err(), ScenarioError::Horizon);
    }

    #[test]
    fn event_limit_must_bracket_requests() {
        let mut x = s();
        x.max_events = 10;
        x.requests = 100;
        assert_eq!(validate(x).unwrap_err(), ScenarioError::EventLimit);
    }

    #[test]
    fn negative_weight_cardinality_mismatch_is_rejected() {
        let mut x = s();
        x.tenant_weights = Some(vec![1, 1]);
        assert_eq!(validate(x).unwrap_err(), ScenarioError::TenantWeights);
    }

    #[test]
    fn zero_weight_is_rejected() {
        let mut x = s();
        x.tenant_weights = Some(vec![1, 0, 1, 1]);
        assert_eq!(validate(x).unwrap_err(), ScenarioError::TenantWeights);
    }

    #[test]
    fn overflowing_weight_sum_is_rejected() {
        let mut x = s();
        x.tenant_weights = Some(vec![u32::MAX, u32::MAX, 1, 1]);
        assert_eq!(validate(x).unwrap_err(), ScenarioError::TenantWeights);
    }

    #[test]
    fn version_must_be_one() {
        let mut x = s();
        x.version = 2;
        assert_eq!(validate(x).unwrap_err(), ScenarioError::Version);
    }

    #[test]
    fn digest_is_stable_across_identical_scenarios() {
        assert_eq!(scenario_digest(&s()), scenario_digest(&s()));
    }

    #[test]
    fn digest_changes_with_weights() {
        let mut weighted = s();
        weighted.tenant_weights = Some(vec![4, 1, 1, 1]);
        assert_ne!(scenario_digest(&s()), scenario_digest(&weighted));
    }

    #[test]
    fn weighted_tenant_assignment_matches_cycle() {
        let mut x = s();
        x.tenant_weights = Some(vec![2, 1, 1, 1]);
        let v = validate(x).unwrap();
        // cycle length 5: tenant 0 gets ids 0-1, tenant1 id2, tenant2 id3, tenant3 id4
        assert_eq!(v.tenant_for_request(0), 0);
        assert_eq!(v.tenant_for_request(1), 0);
        assert_eq!(v.tenant_for_request(2), 1);
        assert_eq!(v.tenant_for_request(3), 2);
        assert_eq!(v.tenant_for_request(4), 3);
        assert_eq!(v.tenant_for_request(5), 0);
    }

    #[test]
    fn unweighted_tenant_assignment_is_round_robin() {
        let v = validate(s()).unwrap();
        assert_eq!(v.tenant_for_request(0), 0);
        assert_eq!(v.tenant_for_request(5), 1);
    }
}
