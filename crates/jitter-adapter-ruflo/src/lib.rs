//! Infrastructure Adapter: `RufloSignalAdapter` implements `HarnessSink`.
//!
//! ADR-0114 (Orchestration Signal Boundary); docs/ddd/12-operations-failure.md.
//!
//! **Honesty note (docs/integration-matrix.md, Ruflo row)**: no Ruflo client
//! library exists to integrate against — the only Ruflo surface in this repository
//! is an MCP coordination/provenance channel, not a Rust dependency, and no source
//! revision or package version has ever been pinned for it. This adapter therefore
//! implements `HarnessSink` with a real, thread-safe interruption-signal primitive
//! that *any* future Ruflo transport (an MCP listener, a webhook handler) can drive by
//! calling [`RufloSignalAdapter::signal_interruption`] — it is not a stub, but it is
//! also not a network client, because there is nothing real to speak to yet.

use jitter_outcome::OutcomeEvent;
use jitter_ports::{HarnessSink, InterruptionSignal, PortError};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};

pub struct RufloSignalAdapter {
    interrupt_at: AtomicI64,
    ticks_seen: AtomicU64,
}

impl Default for RufloSignalAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl RufloSignalAdapter {
    pub fn new() -> Self {
        Self {
            interrupt_at: AtomicI64::new(-1),
            ticks_seen: AtomicU64::new(0),
        }
    }

    /// Signals that the run should be interrupted after `at_tick`. Called by whatever
    /// real coordination transport eventually backs this adapter.
    pub fn signal_interruption(&self, at_tick: u32) {
        self.interrupt_at
            .store(i64::from(at_tick), Ordering::SeqCst);
    }

    pub fn clear(&self) {
        self.interrupt_at.store(-1, Ordering::SeqCst);
    }

    pub fn ticks_seen(&self) -> u64 {
        self.ticks_seen.load(Ordering::SeqCst)
    }
}

impl HarnessSink for RufloSignalAdapter {
    fn on_tick(&self, _event: &OutcomeEvent) -> Result<(), PortError> {
        self.ticks_seen.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn poll_interruption(&self) -> Option<InterruptionSignal> {
        let at = self.interrupt_at.load(Ordering::SeqCst);
        (at >= 0).then_some(InterruptionSignal { at_tick: at as u32 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_interruption_by_default() {
        let adapter = RufloSignalAdapter::new();
        assert_eq!(adapter.poll_interruption(), None);
    }

    #[test]
    fn signal_then_clear_round_trips() {
        let adapter = RufloSignalAdapter::new();
        adapter.signal_interruption(6);
        assert_eq!(
            adapter.poll_interruption(),
            Some(InterruptionSignal { at_tick: 6 })
        );
        adapter.clear();
        assert_eq!(adapter.poll_interruption(), None);
    }

    #[test]
    fn on_tick_counts_every_report() {
        let adapter = RufloSignalAdapter::new();
        let event = OutcomeEvent::Admitted {
            tenant_id: 0,
            request_id: 0,
            tick: 0,
        };
        adapter.on_tick(&event).unwrap();
        adapter.on_tick(&event).unwrap();
        assert_eq!(adapter.ticks_seen(), 2);
    }

    #[test]
    fn simulated_interruption_at_tick_six_matches_architecture_scenario() {
        let adapter = RufloSignalAdapter::new();
        adapter.signal_interruption(6);
        let signal = adapter
            .poll_interruption()
            .expect("interruption must be reported");
        assert_eq!(signal.at_tick, 6);
    }

    #[test]
    fn passes_shared_harness_sink_contract() {
        let adapter = RufloSignalAdapter::new();
        adapter.signal_interruption(6);
        jitter_contract_tests::assert_harness_sink(&adapter);
    }
}
