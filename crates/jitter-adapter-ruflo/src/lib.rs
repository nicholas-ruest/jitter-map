//! Ruflo coordination adapter and a bounded live CLI transport.
//!
//! ADR-0114 and ADR-0123; docs/ddd/12-operations-failure.md.

use jitter_outcome::OutcomeEvent;
use jitter_ports::{HarnessSink, InterruptionSignal, PortError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, Instant};

const MAX_OUTPUT_BYTES: usize = 64 * 1024;

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
}

fn canonical_file(path: &Path) -> Result<PathBuf, PortError> {
    let canonical = path
        .canonicalize()
        .map_err(|error| PortError::Unavailable(format!("executable unavailable: {error}")))?;
    if !canonical.is_file() {
        return Err(PortError::Unavailable("executable is not a file".into()));
    }
    Ok(canonical)
}

fn canonical_dir(path: &Path) -> Result<PathBuf, PortError> {
    let canonical = path.canonicalize().map_err(|error| {
        PortError::Unavailable(format!("working directory unavailable: {error}"))
    })?;
    if !canonical.is_dir() {
        return Err(PortError::Unavailable(
            "working directory is not a directory".into(),
        ));
    }
    Ok(canonical)
}

fn run_bounded(
    program: &Path,
    args: &[&str],
    working_dir: &Path,
    timeout: Duration,
) -> Result<Vec<u8>, PortError> {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(working_dir)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| PortError::Unavailable(format!("Ruflo spawn failed: {error}")))?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let stdout_reader = thread::spawn(move || read_stream(stdout));
    let stderr_reader = thread::spawn(move || read_stream(stderr));
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| PortError::Unavailable(format!("Ruflo wait failed: {error}")))?
        {
            break status;
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PortError::Unavailable("Ruflo process timed out".into()));
        }
        thread::sleep(Duration::from_millis(10));
    };
    let stdout = join_reader(stdout_reader)?;
    let stderr = join_reader(stderr_reader)?;
    if stdout.len() > MAX_OUTPUT_BYTES || stderr.len() > MAX_OUTPUT_BYTES {
        return Err(PortError::Rejected("Ruflo output exceeded 64 KiB".into()));
    }
    if !status.success() {
        return Err(PortError::Unavailable(format!(
            "Ruflo exited {status}: {}",
            String::from_utf8_lossy(&stderr).trim()
        )));
    }
    Ok(stdout)
}

fn read_stream(mut stream: impl Read) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    stream
        .by_ref()
        .take((MAX_OUTPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn join_reader(handle: thread::JoinHandle<std::io::Result<Vec<u8>>>) -> Result<Vec<u8>, PortError> {
    handle
        .join()
        .map_err(|_| PortError::Unavailable("Ruflo reader panicked".into()))?
        .map_err(|error| PortError::Unavailable(format!("Ruflo output read failed: {error}")))
}

fn append_receipt(path: &Path, receipt: &RufloRegistryReceipt) -> Result<(), PortError> {
    let line = serde_json::to_string(receipt)
        .map_err(|error| PortError::Rejected(format!("receipt serialization failed: {error}")))?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| PortError::Unavailable(format!("receipt open failed: {error}")))?;
    writeln!(file, "{line}")
        .map_err(|error| PortError::Unavailable(format!("receipt write failed: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
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
    #[ignore = "requires the pinned Ruflo CLI and live registry"]
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
    }
}
