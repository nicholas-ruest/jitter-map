//! Deterministic retry schedule simulation and collision analysis.

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;

use serde::Serialize;

/// Hard guardrail against accidentally allocating an unbounded event plan.
pub const MAX_EVENTS: u64 = 5_000_000;

/// Retry delay strategy to simulate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Uniformly sample from zero through the exponential ceiling.
    Full,
    /// Sample from half the exponential ceiling through the ceiling.
    Equal,
    /// Sample from the base delay through three times the previous delay.
    Decorrelated,
    /// Use the exponential ceiling without jitter.
    None,
}

impl Strategy {
    /// Stable lowercase name used by human-readable output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Equal => "equal",
            Self::Decorrelated => "decorrelated",
            Self::None => "none",
        }
    }
}

/// Inputs for a retry simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PlanConfig {
    pub clients: u32,
    pub attempts: u32,
    pub base_ms: u64,
    pub cap_ms: u64,
    pub strategy: Strategy,
    pub seed: u64,
    pub collision_window_ms: u64,
}

/// One scheduled retry for one client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetryEvent {
    pub client: u32,
    pub attempt: u32,
    pub delay_ms: u64,
    pub wake_at_ms: u64,
}

/// Aggregate collision metrics for a retry plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlanSummary {
    pub total_events: u64,
    pub occupied_windows: u64,
    pub colliding_client_windows: u64,
    pub collision_pairs: u64,
    pub peak_window_load: u64,
}

/// Full deterministic simulation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RetryPlan {
    pub config: PlanConfig,
    pub events: Vec<RetryEvent>,
    pub summary: PlanSummary,
}

/// Validation failures for an invalid or unsafe plan request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    ZeroClients,
    ZeroAttempts,
    ZeroBase,
    ZeroCap,
    BaseExceedsCap,
    ZeroCollisionWindow,
    TooManyEvents { requested: u64, maximum: u64 },
}

impl fmt::Display for PlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroClients => formatter.write_str("clients must be greater than zero"),
            Self::ZeroAttempts => formatter.write_str("attempts must be greater than zero"),
            Self::ZeroBase => formatter.write_str("base delay must be greater than zero"),
            Self::ZeroCap => formatter.write_str("delay cap must be greater than zero"),
            Self::BaseExceedsCap => formatter.write_str("base delay cannot exceed the cap"),
            Self::ZeroCollisionWindow => {
                formatter.write_str("collision window must be greater than zero")
            }
            Self::TooManyEvents { requested, maximum } => write!(
                formatter,
                "requested {requested} events, which exceeds the safety limit of {maximum}"
            ),
        }
    }
}

impl Error for PlanError {}

/// Generate a deterministic retry plan and its collision summary.
///
/// A collision window counts each client at most once, even if zero-delay retries
/// place more than one of that client's attempts inside the same window.
pub fn generate_plan(config: PlanConfig) -> Result<RetryPlan, PlanError> {
    validate(config)?;

    let total_events = u64::from(config.clients) * u64::from(config.attempts);
    let mut events = Vec::with_capacity(total_events as usize);

    for client in 0..config.clients {
        let mut wake_at_ms = 0_u64;
        let mut ceiling_ms = config.base_ms;
        let mut previous_delay_ms = config.base_ms;

        for attempt in 0..config.attempts {
            let random = random_word(config.seed, client, attempt);
            let delay_ms = match config.strategy {
                Strategy::Full => sample_inclusive(random, 0, ceiling_ms),
                Strategy::Equal => sample_inclusive(random, ceiling_ms / 2, ceiling_ms),
                Strategy::Decorrelated if attempt == 0 => config.base_ms,
                Strategy::Decorrelated => {
                    let upper = previous_delay_ms.saturating_mul(3).min(config.cap_ms);
                    sample_inclusive(random, config.base_ms, upper)
                }
                Strategy::None => ceiling_ms,
            };

            wake_at_ms = wake_at_ms.saturating_add(delay_ms);
            events.push(RetryEvent {
                client,
                attempt: attempt + 1,
                delay_ms,
                wake_at_ms,
            });
            previous_delay_ms = delay_ms;
            ceiling_ms = ceiling_ms.saturating_mul(2).min(config.cap_ms);
        }
    }

    events.sort_by_key(|event| (event.wake_at_ms, event.client, event.attempt));
    let summary = summarize(&events, config.collision_window_ms);

    Ok(RetryPlan {
        config,
        events,
        summary,
    })
}

fn validate(config: PlanConfig) -> Result<(), PlanError> {
    if config.clients == 0 {
        return Err(PlanError::ZeroClients);
    }
    if config.attempts == 0 {
        return Err(PlanError::ZeroAttempts);
    }
    if config.base_ms == 0 {
        return Err(PlanError::ZeroBase);
    }
    if config.cap_ms == 0 {
        return Err(PlanError::ZeroCap);
    }
    if config.base_ms > config.cap_ms {
        return Err(PlanError::BaseExceedsCap);
    }
    if config.collision_window_ms == 0 {
        return Err(PlanError::ZeroCollisionWindow);
    }

    let requested = u64::from(config.clients) * u64::from(config.attempts);
    if requested > MAX_EVENTS {
        return Err(PlanError::TooManyEvents {
            requested,
            maximum: MAX_EVENTS,
        });
    }

    Ok(())
}

fn summarize(events: &[RetryEvent], window_ms: u64) -> PlanSummary {
    let mut clients_by_window: HashMap<u64, HashSet<u32>> = HashMap::new();
    for event in events {
        clients_by_window
            .entry(event.wake_at_ms / window_ms)
            .or_default()
            .insert(event.client);
    }

    let mut colliding_client_windows = 0_u64;
    let mut collision_pairs = 0_u64;
    let mut peak_window_load = 0_u64;

    for clients in clients_by_window.values() {
        let load = clients.len() as u64;
        peak_window_load = peak_window_load.max(load);
        if load > 1 {
            colliding_client_windows += load;
            collision_pairs += load * (load - 1) / 2;
        }
    }

    PlanSummary {
        total_events: events.len() as u64,
        occupied_windows: clients_by_window.len() as u64,
        colliding_client_windows,
        collision_pairs,
        peak_window_load,
    }
}

fn random_word(seed: u64, client: u32, attempt: u32) -> u64 {
    let mixed = seed
        ^ u64::from(client).wrapping_mul(0x9e37_79b9_7f4a_7c15)
        ^ u64::from(attempt).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    splitmix64(mixed)
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

fn sample_inclusive(random: u64, low: u64, high: u64) -> u64 {
    debug_assert!(low <= high);
    let span = u128::from(high) - u128::from(low) + 1;
    let offset = (u128::from(random) * span) >> 64;
    low + offset as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(strategy: Strategy) -> PlanConfig {
        PlanConfig {
            clients: 3,
            attempts: 4,
            base_ms: 100,
            cap_ms: 800,
            strategy,
            seed: 42,
            collision_window_ms: 10,
        }
    }

    #[test]
    fn same_seed_produces_identical_plan() {
        let first = generate_plan(config(Strategy::Full)).unwrap();
        let second = generate_plan(config(Strategy::Full)).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn changing_seed_changes_jittered_plan() {
        let first = generate_plan(config(Strategy::Full)).unwrap();
        let mut changed = config(Strategy::Full);
        changed.seed += 1;
        let second = generate_plan(changed).unwrap();
        assert_ne!(first.events, second.events);
    }

    #[test]
    fn no_jitter_uses_capped_exponential_delays() {
        let plan = generate_plan(PlanConfig {
            clients: 1,
            attempts: 5,
            base_ms: 100,
            cap_ms: 500,
            strategy: Strategy::None,
            seed: 7,
            collision_window_ms: 1,
        })
        .unwrap();
        let delays: Vec<u64> = plan.events.iter().map(|event| event.delay_ms).collect();
        assert_eq!(delays, vec![100, 200, 400, 500, 500]);
    }

    #[test]
    fn equal_jitter_stays_in_upper_half() {
        let plan = generate_plan(config(Strategy::Equal)).unwrap();
        for event in &plan.events {
            let ceiling = (100_u64 << (event.attempt - 1)).min(800);
            assert!(event.delay_ms >= ceiling / 2);
            assert!(event.delay_ms <= ceiling);
        }
    }

    #[test]
    fn decorrelated_jitter_starts_at_base_and_respects_cap() {
        let plan = generate_plan(config(Strategy::Decorrelated)).unwrap();
        for client in 0..3 {
            let client_events: Vec<_> = plan
                .events
                .iter()
                .filter(|event| event.client == client)
                .collect();
            assert_eq!(client_events[0].delay_ms, 100);
            assert!(client_events.iter().all(|event| event.delay_ms <= 800));
        }
    }

    #[test]
    fn no_jitter_reports_all_clients_colliding() {
        let plan = generate_plan(config(Strategy::None)).unwrap();
        assert_eq!(plan.summary.peak_window_load, 3);
        assert_eq!(plan.summary.collision_pairs, 12);
        assert_eq!(plan.summary.colliding_client_windows, 12);
    }

    #[test]
    fn rejects_invalid_ranges_and_unbounded_requests() {
        let mut invalid = config(Strategy::Full);
        invalid.base_ms = 900;
        assert_eq!(generate_plan(invalid), Err(PlanError::BaseExceedsCap));

        invalid = config(Strategy::Full);
        invalid.clients = 5_000_001;
        invalid.attempts = 1;
        assert!(matches!(
            generate_plan(invalid),
            Err(PlanError::TooManyEvents { .. })
        ));
    }

    #[test]
    fn inclusive_sampler_handles_entire_u64_range() {
        assert_eq!(sample_inclusive(0, 0, u64::MAX), 0);
        assert_eq!(sample_inclusive(u64::MAX, 0, u64::MAX), u64::MAX);
    }
}
