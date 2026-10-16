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
struct Registration {
    enabled: bool,
}

impl RufloProcessTransport {
    pub fn new(
        program: impl AsRef<Path>,
        working_dir: impl AsRef<Path>,
        receipt_path: impl AsRef<Path>,
    ) -> Self {
        Self {
            program: program.as_ref().to_path_buf(),
            working_dir: working_dir.as_ref().to_path_buf(),
            receipt_path: receipt_path.as_ref().to_path_buf(),
            timeout: Duration::from_secs(30),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Executes the pinned Ruflo CLI registry boundary without invoking a shell.
    pub fn probe_registry(&self) -> Result<RufloRegistryReceipt, PortError> {
        let program = canonical_file(&self.program)?;
        let working_dir = canonical_dir(&self.working_dir)?;
        let started = Instant::now();
        let output = run_bounded(
            &program,
            &["federation", "registry", "--format", "json"],
            &working_dir,
            self.timeout,
        )?;
        let registry: RegistryResponse = serde_json::from_slice(&output).map_err(|error| {
            PortError::Rejected(format!("invalid Ruflo registry JSON: {error}"))
        })?;
        if !registry.relay.starts_with("wss://") || !registry.http_base.starts_with("https://") {
            return Err(PortError::Rejected(
                "Ruflo registry returned non-TLS endpoints".into(),
            ));
        }
        if registry.swarm_tag.trim().is_empty() {
            return Err(PortError::Rejected("Ruflo swarm tag is empty".into()));
        }
        let receipt = RufloRegistryReceipt {
            package: "@claude-flow/cli@3.56.10".into(),
            executable: program.display().to_string(),
            relay: registry.relay,
            http_base: registry.http_base,
            swarm_tag: registry.swarm_tag,
            registration_enabled: registry.registration.enabled,
            argv_sha256: format!(
                "{:x}",
                Sha256::digest(b"federation\0registry\0--format\0json")
            ),
            output_sha256: format!("{:x}", Sha256::digest(&output)),
            exit_status: 0,
            elapsed_ms: started.elapsed().as_millis(),
            authority: "none".into(),
        };
        append_receipt(&self.receipt_path, &receipt)?;
        Ok(receipt)
    }

    /// Reads the public federation through the pinned CLI without membership authority.
    pub fn sync_federation(
        &self,
        since_secs: u32,
        limit: u16,
    ) -> Result<RufloSyncReceipt, PortError> {
        if !(1..=2_592_000).contains(&since_secs) {
            return Err(PortError::Rejected(
                "Ruflo sync since_secs must be between 1 and 2592000".into(),
            ));
        }
        if !(1..=1_000).contains(&limit) {
            return Err(PortError::Rejected(
                "Ruflo sync limit must be between 1 and 1000".into(),
            ));
        }
        let program = canonical_file(&self.program)?;
        let working_dir = canonical_dir(&self.working_dir)?;
        let since = since_secs.to_string();
        let limit_value = limit.to_string();
        let args = [
            "federation",
            "sync",
            "--since",
            since.as_str(),
            "--limit",
            limit_value.as_str(),
            "--format",
            "json",
        ];
        let started = Instant::now();
        let output = run_bounded(&program, &args, &working_dir, self.timeout)?;
        let (envelope, message_types) = parse_federation(&output)?;
        let argv_bytes = args.join("\0");
        let receipt = RufloSyncReceipt {
            package: "@claude-flow/cli@3.56.10".into(),
            executable: program.display().to_string(),
            relay: envelope.relay,
            provenance: envelope.provenance,
            retrieved_at: envelope.retrieved_at,
            since_secs,
            limit,
            message_count: envelope.data.count,
            message_types,
            argv_sha256: format!("{:x}", Sha256::digest(argv_bytes.as_bytes())),
            output_sha256: format!("{:x}", Sha256::digest(&output)),
            exit_status: 0,
            elapsed_ms: started.elapsed().as_millis(),
            authority: "none".into(),
        };
        append_receipt(&self.receipt_path, &receipt)?;
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use process::MAX_OUTPUT_BYTES;
    use std::io::Write;
    use std::thread;
    use tempfile::tempdir;

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
    fn missing_executable_fails_closed() {
        let dir = tempdir().unwrap();
        let transport = RufloProcessTransport::new(
            dir.path().join("missing"),
            dir.path(),
            dir.path().join("receipt.jsonl"),
        );
        assert!(matches!(
            transport.probe_registry(),
            Err(PortError::Unavailable(_))
        ));
    }

    #[test]
    fn timeout_fails_closed() {
        let executable = std::env::current_exe().unwrap();
        let dir = tempdir().unwrap();
        let error = run_bounded(
            &executable,
            &[
                "--ignored",
                "--exact",
                "tests::fixture_timeout",
                "--nocapture",
            ],
            dir.path(),
            Duration::from_millis(30),
        )
        .unwrap_err();
        assert!(matches!(error, PortError::Unavailable(message) if message.contains("timed out")));
    }

    #[test]
    fn non_zero_exit_fails_closed() {
        let executable = std::env::current_exe().unwrap();
        let dir = tempdir().unwrap();
        let error = run_bounded(
            &executable,
            &[
                "--ignored",
                "--exact",
                "tests::fixture_non_zero",
                "--nocapture",
            ],
            dir.path(),
            Duration::from_secs(2),
        )
        .unwrap_err();
        assert!(matches!(error, PortError::Unavailable(message) if message.contains("exited")));
    }

    #[test]
    fn oversized_output_fails_closed() {
        let executable = std::env::current_exe().unwrap();
        let dir = tempdir().unwrap();
        let error = run_bounded(
            &executable,
            &[
                "--ignored",
                "--exact",
                "tests::fixture_oversized",
                "--nocapture",
            ],
            dir.path(),
            Duration::from_secs(2),
        )
        .unwrap_err();
        assert!(matches!(error, PortError::Rejected(message) if message.contains("64 KiB")));
    }

    #[test]
    fn malformed_registry_schema_is_rejected() {
        assert!(serde_json::from_slice::<RegistryResponse>(br#"{"relay":"wss://relay"}"#).is_err());
    }

    #[test]
    fn federation_sync_preserves_untrusted_provenance_and_counts_types() {
        let output = br#"{"untrusted":true,"provenance":"third-party data","relay":"wss://relay.example","retrievedAt":"2026-10-16T00:00:00Z","data":{"count":3,"messages":[{"type":"Result","body":"opaque"},{"type":"Result"},{"type":"Announcement"}]}}"#;
        let (envelope, types) = parse_federation(output).unwrap();
        assert_eq!(envelope.data.count, 3);
        assert_eq!(types["Result"], 2);
        assert_eq!(types["Announcement"], 1);
    }

    #[test]
    fn federation_sync_accepts_zero_message_envelope() {
        let output = br#"{"untrusted":true,"provenance":"third-party data","relay":"wss://relay.example","retrievedAt":"2026-10-16T00:00:00Z","data":{"count":0,"messages":[]}}"#;
        let (envelope, types) = parse_federation(output).unwrap();
        assert_eq!(envelope.data.count, 0);
        assert!(types.is_empty());
    }

    #[test]
    fn federation_sync_rejects_missing_untrusted_label() {
        let output = br#"{"untrusted":false,"provenance":"third-party data","relay":"wss://relay.example","retrievedAt":"2026-10-16T00:00:00Z","data":{"count":0,"messages":[]}}"#;
        let error = parse_federation(output).unwrap_err();
        assert!(matches!(error, PortError::Rejected(message) if message.contains("untrusted")));
    }

    #[test]
    fn federation_sync_rejects_count_mismatch() {
        let output = br#"{"untrusted":true,"provenance":"third-party data","relay":"wss://relay.example","retrievedAt":"2026-10-16T00:00:00Z","data":{"count":2,"messages":[{"type":"Result"}]}}"#;
        let error = parse_federation(output).unwrap_err();
        assert!(matches!(error, PortError::Rejected(message) if message.contains("count")));
    }

    #[test]
    fn federation_sync_rejects_invalid_message_type() {
        let output = br#"{"untrusted":true,"provenance":"third-party data","relay":"wss://relay.example","retrievedAt":"2026-10-16T00:00:00Z","data":{"count":1,"messages":[{"body":"opaque"}]}}"#;
        let error = parse_federation(output).unwrap_err();
        assert!(matches!(error, PortError::Rejected(message) if message.contains("type")));
    }

    #[test]
    fn federation_sync_classifies_signed_raw_wrapper_as_opaque() {
        let output = br#"{"untrusted":true,"provenance":"third-party data","relay":"wss://relay.example","retrievedAt":"2026-10-16T00:00:00Z","data":{"count":1,"messages":[{"id":"abc","pubkey":"key","raw":"opaque signed payload"}]}}"#;
        let (envelope, types) = parse_federation(output).unwrap();
        assert_eq!(envelope.data.count, 1);
        assert_eq!(types["opaque-raw"], 1);
    }

    #[test]
    fn federation_sync_rejects_invalid_bounds_before_process_access() {
        let dir = tempdir().unwrap();
        let transport = RufloProcessTransport::new(
            dir.path().join("missing"),
            dir.path(),
            dir.path().join("receipt.jsonl"),
        );
        assert!(matches!(
            transport.sync_federation(0, 1),
            Err(PortError::Rejected(_))
        ));
        assert!(matches!(
            transport.sync_federation(1, 0),
            Err(PortError::Rejected(_))
        ));
    }

    #[test]
    #[ignore = "child-process fixture only"]
    fn fixture_timeout() {
        thread::sleep(Duration::from_secs(5));
    }

    #[test]
    #[ignore = "child-process fixture only"]
    fn fixture_non_zero() {
        std::process::exit(9);
    }

    #[test]
    #[ignore = "child-process fixture only"]
    fn fixture_oversized() {
        std::io::stdout()
            .write_all(&vec![b'x'; MAX_OUTPUT_BYTES + 1])
            .unwrap();
    }

    #[test]
    fn passes_shared_harness_sink_contract() {
        let adapter = RufloSignalAdapter::new();
        adapter.signal_interruption(6);
        jitter_contract_tests::assert_harness_sink(&adapter);
    }

    #[test]
    #[ignore = "requires the pinned Ruflo CLI and live federation"]
    fn live_ruflo_registry_when_configured() {
        let program = std::env::var_os("JITTER_RUFLO_BIN").expect("JITTER_RUFLO_BIN");
        let workdir = std::env::var_os("JITTER_RUFLO_WORKDIR").expect("JITTER_RUFLO_WORKDIR");
        let dir = tempdir().unwrap();
        let receipt = RufloProcessTransport::new(program, workdir, dir.path().join("ruflo.jsonl"))
            .probe_registry()
            .unwrap();
        assert_eq!(receipt.package, "@claude-flow/cli@3.56.10");
        assert_eq!(receipt.authority, "none");
        assert!(receipt.registration_enabled);

        let sync = RufloProcessTransport::new(
            std::env::var_os("JITTER_RUFLO_BIN").expect("JITTER_RUFLO_BIN"),
            std::env::var_os("JITTER_RUFLO_WORKDIR").expect("JITTER_RUFLO_WORKDIR"),
            dir.path().join("ruflo.jsonl"),
        )
        .sync_federation(604_800, 20)
        .unwrap();
        assert!(sync.message_count <= 20);
        assert_eq!(
            sync.message_types.values().sum::<u64>(),
            sync.message_count as u64
        );
        assert_eq!(sync.authority, "none");
    }
}
