//! Ruflo coordination adapter and a bounded live CLI transport.
//!
//! ADR-0114, ADR-0123, and ADR-0124; docs/ddd/12-operations-failure.md.

mod federation;
mod process;

use jitter_outcome::OutcomeEvent;
use jitter_ports::{HarnessSink, InterruptionSignal, PortError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use federation::parse_federation;
use process::{append_receipt, canonical_dir, canonical_file, run_bounded};

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

#[derive(Debug, Clone)]
pub struct RufloProcessTransport {
    program: PathBuf,
    working_dir: PathBuf,
    receipt_path: PathBuf,
    timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RufloRegistryReceipt {
    pub package: String,
    pub executable: String,
    pub relay: String,
    pub http_base: String,
    pub swarm_tag: String,
    pub registration_enabled: bool,
    pub argv_sha256: String,
    pub output_sha256: String,
    pub exit_status: i32,
    pub elapsed_ms: u128,
    pub authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RufloSyncReceipt {
    pub package: String,
    pub executable: String,
    pub relay: String,
    pub provenance: String,
    pub retrieved_at: String,
    pub since_secs: u32,
    pub limit: u16,
    pub message_count: usize,
    pub message_types: BTreeMap<String, u64>,
    pub argv_sha256: String,
    pub output_sha256: String,
    pub exit_status: i32,
    pub elapsed_ms: u128,
    pub authority: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegistryResponse {
    relay: String,
    http_base: String,
    swarm_tag: String,
    registration: Registration,
}

#[derive(Debug, Deserialize)]
struct Rh∫⁄n∂⁄Óù∆≠y