//! Domain Core: deterministic generation of the initial retry-request arrival trace.
//!
//! ADR-0102 (Deterministic Generation); docs/ddd/03-scenario-model.md.
//!
//! `generate` is infallible given a `ValidatedScenario` — determinism is the contract,
//! not error handling (docs/implementation-contract.md). The `seed` is carried on the
//! resulting [`WorkloadTrace`] rather than consumed here: the initial arrival of every
//! request at tick 0 is a pure function of tenant demand weights, not of `seed` — the
//! seed instead drives the reproducible jitter delay computed downstream by
//! `jitter-policy` once a retry is needed. Keeping the seed attached to the trace (not
//! dropped) is what lets `jitter-simulation` thread one seed through both stages without
//! a side channel.

use jitter_scenario::ValidatedScenario;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryRequest {
    pub request_id: u32,
    pub tenant_id: u32,
    pub tick: u32,
    pub attempt: u32,
}

/// The deterministic trace of initial request arrivals for one scenario, plus the seed
/// that must be used for every subsequent jitter computation over this trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadTrace {
    pub seed: u64,
    requests: Vec<RetryRequest>,
}

impl WorkloadTrace {
    pub fn seed(&self) -> u64 {
        self.seed
    }

    pub fn requests(&self) -> &[RetryRequest] {
        &self.requests
    }
}

impl IntoIterator for WorkloadTrace {
    type Item = RetryRequest;
    type IntoIter = std::vec::IntoIter<RetryRequest>;

    fn into_iter(self) -> Self::IntoIter {
        self.requests.into_iter()
    }
}

/// Generates the tick-0 arrival of every request in `scenario`, assigned to tenants by
/// the scenario's demand-weight cycle. Two independent calls with the same scenario and
/// seed produce a byte-identical serialized trace (row 2's acceptance criterion).
pub fn generate(scenario: &ValidatedScenario, seed: u64) -> WorkloadTrace {
    let requests = (0..scenario.requests())
        .map(|id| RetryRequest {
            request_id: id,
            tenant_id: scenario.tenant_for_request(id),
            tick: 0,
            attempt: 1,
        })
        .collect();
    WorkloadTrace { seed, requests }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jitter_scenario::{FailureClass, Scenario, validate};

    fn scenario() -> ValidatedScenario {
        validate(Scenario {
            version: 1,
            requests: 20,
            tenants: 4,
            capacity_per_tick: 5,
            outage_ticks: 2,
            horizon_ticks: 50,
            seed: 7,
            failure: FailureClass::Throttling,
            tenant_weights: Some(vec![4, 1, 1, 1]),
            max_events: 10_000,
        })
        .unwrap()
    }

    #[test]
    fn determinism_same_seed_same_trace_bytes() {
        let a = generate(&scenario(), 42);
        let b = generate(&scenario(), 42);
        assert_eq!(
            serde_json::to_string(&a.requests()).unwrap(),
            serde_json::to_string(&b.requests()).unwrap()
        );
        assert_eq!(a.seed(), b.seed());
    }

    #[test]
    fn requests_count_matches_scenario() {
        let trace = generate(&scenario(), 1);
        assert_eq!(trace.requests().len(), 20);
    }

    #[test]
    fn tenant_assignment_follows_demand_weights() {
        let trace = generate(&scenario(), 1);
        let per_tenant = trace.requests().iter().fold([0u32; 4], |mut acc, r| {
            acc[r.tenant_id as usize] += 1;
            acc
        });
        // cycle [4,1,1,1] sums to 7; 20 requests -> tenant0 gets 4 of every 7
        assert_eq!(per_tenant[0], 12);
        assert_eq!(per_tenant[1] + per_tenant[2] + per_tenant[3], 8);
    }

    #[test]
    fn all_requests_start_at_tick_zero_attempt_one() {
        let trace = generate(&scenario(), 1);
        assert!(
            trace
                .requests()
                .iter()
                .all(|r| r.tick == 0 && r.attempt == 1)
        );
    }
}
