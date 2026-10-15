//! Append-only evaluation feed plus a bounded live MetaHarness/Flywheel runner.
//!
//! ADR-0115 and ADR-0123; docs/ddd/09-upstream-integrations.md.

use jitter_ports::{CandidateResult, FlywheelSink, PortError};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const MAX_OUTPUT_BYTES: usize = 64 * 1024;

pub struct MetaharnessAdapter {
    feed_path: PathBuf,
}

impl MetaharnessAdapter {
    pub fn new(feed_path: impl AsRef<Path>) -> Self {
        Self {
            feed_path: feed_path.as_ref().to_path_buf(),
        }
    }
}

impl FlywheelSink for MetaharnessAdapter {
    fn submit(&self, result: &CandidateResult) -> Result<(), PortError> {
        let line = serde_json::to_string(result)
            .map_err(|error| PortError::Rejected(error.to_string()))?;
        append_line(&self.feed_path, &line)
    }
}

#[derive(Debug, Clone)]
pub struct MetaharnessProcessRunner {
    config: MetaharnessProcessConfig,
    receipt_path: PathBuf,
    timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct MetaharnessProcessConfig {
    node_program: PathBuf,
    bridge_script: PathBuf,
    darwin_cli: PathBuf,
    darwin_genome: PathBuf,
    candidate_binary: PathBuf,
    repository: PathBuf,
    flywheel_entry: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaharnessReceipt {
    pub darwin: DarwinReceipt,
    pub package: String,
    pub generations_run: u64,
    pub promotions: u64,
    pub replay_verdict: ReplayVerdict,
    pub output_sha256: String,
    pub exit_status: i32,
    pub authority: String,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DarwinReceipt {
    pub package: String,
    pub winner: String,
    pub argv_sha256: String,
    pub input_sha256: String,
    pub output_sha256: String,
    pub exit_status: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayVerdict {
    pub pass: bool,
    #[serde(flatten)]
    pub details: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BridgeResponse {
    package: String,
    generations_run: u64,
    promotions: u64,
    replay_verdict: ReplayVerdict,
    authority: String,
}

impl MetaharnessProcessConfig {
    pub fn new(
        node_program: impl AsRef<Path>,
        bridge_script: impl AsRef<Path>,
        darwin_cli: impl AsRef<Path>,
        darwin_genome: impl AsRef<Path>,
        candidate_binary: impl AsRef<Path>,
        repository: impl AsRef<Path>,
        flywheel_entry: impl AsRef<Path>,
    ) -> Self {
        Self {
            node_program: node_program.as_ref().to_path_buf(),
            bridge_script: bridge_script.as_ref().to_path_buf(),
            darwin_cli: darwin_cli.as_ref().to_path_buf(),
            darwin_genome: darwin_genome.as_ref().to_path_buf(),
            candidate_binary: candidate_binary.as_ref().to_path_buf(),
            repository: repository.as_ref().to_path_buf(),
            flywheel_entry: flywheel_entry.as_ref().to_path_buf(),
        }
    }
}

impl MetaharnessProcessRunner {
    pub fn new(config: MetaharnessProcessConfig, receipt_path: impl AsRef<Path>) -> Self {
        Self {
            config,
            receipt_path: receipt_path.as_ref().to_path_buf(),
            timeout: Duration::from_secs(60),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Runs the repository-owned bridge against the installed Flywheel ESM entry.
    /// No shell is involved and the child receives no promotion authority.
    pub fn evaluate(&self) -> Result<MetaharnessReceipt, PortError> {
        let node = canonical_file(&self.config.node_program, "Node executable")?;
        let script = canonical_file(&self.config.bridge_script, "MetaHarness bridge")?;
        let darwin_cli = canonical_file(&self.config.darwin_cli, "Darwin CLI")?;
        let darwin_genome = canonical_file(&self.config.darwin_genome, "Darwin genome")?;
        let candidate_binary =
            canonical_file(&self.config.candidate_binary, "candidate evaluator")?;
        let repository = canonical_dir(&self.config.repository, "repository")?;
        let entry = canonical_file(&self.config.flywheel_entry, "Flywheel entry")?;
        let started = Instant::now();
        let darwin = run_darwin(
            &node,
            &darwin_cli,
            &darwin_genome,
            &candidate_binary,
            &repository,
            self.timeout,
        )?;
        let output = run_flywheel(&node, &script, &repository, &entry, self.timeout)?;
        let response: BridgeResponse = serde_json::from_slice(&output).map_err(|error| {
            PortError::Rejected(format!("invalid MetaHarness bridge JSON: {error}"))
        })?;
        if response.package != "@metaharness/flywheel@0.1.12" {
            return Err(PortError::Rejected(format!(
                "unexpected MetaHarness package: {}",
                response.package
            )));
        }
        if response.authority != "none" {
            return Err(PortError::Rejected(
                "MetaHarness authority must be none".into(),
            ));
        }
        if response.promotions != 0 || !response.replay_verdict.pass {
            return Err(PortError::Rejected(
                "MetaHarness replay failed closed or attempted promotion".into(),
            ));
        }
        let receipt = MetaharnessReceipt {
            darwin,
            package: response.package,
            generations_run: response.generations_run,
            promotions: response.promotions,
            replay_verdict: response.replay_verdict,
            output_sha256: format!("{:x}", Sha256::digest(&output)),
            exit_status: 0,
            authority: response.authority,
            elapsed_ms: started.elapsed().as_millis(),
        };
        let line = serde_json::to_string(&receipt).map_err(|error| {
            PortError::Rejected(format!("receipt serialization failed: {error}"))
        })?;
        append_line(&self.receipt_path, &line)?;
        Ok(receipt)
    }
}

fn canonical_file(path: &Path, label: &str) -> Result<PathBuf, PortError> {
    let canonical = path
        .canonicalize()
        .map_err(|error| PortError::Unavailable(format!("{label} unavailable: {error}")))?;
    if !canonical.is_file() {
        return Err(PortError::Unavailable(format!("{label} is not a file")));
    }
    Ok(canonical)
}

fn canonical_dir(path: &Path, label: &str) -> Result<PathBuf, PortError> {
    let canonical = path
        .canonicalize()
        .map_err(|error| PortError::Unavailable(format!("{label} unavailable: {error}")))?;
    if !canonical.is_dir() {
        return Err(PortError::Unavailable(format!(
            "{label} is not a directory"
        )));
    }
    Ok(canonical)
}

fn run_darwin(
    node: &Path,
    cli: &Path,
    genome: &Path,
    candidate: &Path,
    repository: &Path,
    timeout: Duration,
) -> Result<DarwinReceipt, PortError> {
    let mut command = Command::new(node);
    command
        .arg(cli)
        .arg("evolve-numeric")
        .arg(repository)
        .arg("--genome")
        .arg(genome)
        .arg("--evaluator")
        .arg(candidate)
        .arg("--generations")
        .arg("1")
        .arg("--children")
        .arg("2")
        .arg("--concurrency")
        .arg("1")
        .arg("--seed")
        .arg("20261014")
        .current_dir(repository);
    let output = run_command_bounded(command, "Darwin", timeout)?;
    let genome_bytes = std::fs::read(genome)
        .map_err(|error| PortError::Unavailable(format!("Darwin genome read failed: {error}")))?;
    let text = std::str::from_utf8(&output)
        .map_err(|error| PortError::Rejected(format!("Darwin output was not UTF-8: {error}")))?;
    let winner = text
        .lines()
        .find_map(|line| line.strip_prefix("Winner: "))
        .filter(|winner| !winner.trim().is_empty())
        .ok_or_else(|| PortError::Rejected("Darwin output omitted Winner".into()))?;
    Ok(DarwinReceipt {
        package: "@metaharness/darwin@0.10.3".into(),
        winner: winner.trim().into(),
        argv_sha256: format!(
            "{:x}",
            Sha256::digest(
                b"evolve-numeric\x00--genome\x00--evaluator\x00--generations\x001\x00--children\x002\x00--concurrency\x001\x00--seed\x0020261014",
            )
        ),
        input_sha256: format!("{:x}", Sha256::digest(&genome_bytes)),
        output_sha256: format!("{:x}", Sha256::digest(&output)),
        exit_status: 0,
    })
}

fn run_flywheel(
    node: &Path,
    script: &Path,
    repository: &Path,
    entry: &Path,
    timeout: Duration,
) -> Result<Vec<u8>, PortError> {
    let mut command = Command::new(node);
    command
        .arg(script)
        .arg(repository)
        .current_dir(repository)
        .env("METAHARNESS_FLYWHEEL_ENTRY", entry)
        .stdin(Stdio::null());
    run_command_bounded(command, "MetaHarness", timeout)
}

fn run_command_bounded(
    mut command: Command,
    label: &str,
    timeout: Duration,
) -> Result<Vec<u8>, PortError> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| PortError::Unavailable(format!("{label} spawn failed: {error}")))?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stderr = child.stderr.take().expect("piped stderr");
    let stdout_reader = thread::spawn(move || read_stream(stdout));
    let stderr_reader = thread::spawn(move || read_stream(stderr));
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| PortError::Unavailable(format!("{label} wait failed: {error}")))?
        {
            break status;
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            return Err(PortError::Unavailable(format!("{label} process timed out")));
        }
        thread::sleep(Duration::from_millis(10));
    };
    let stdout = join_reader(stdout_reader)?;
    let stderr = join_reader(stderr_reader)?;
    if stdout.len() > MAX_OUTPUT_BYTES || stderr.len() > MAX_OUTPUT_BYTES {
        return Err(PortError::Rejected(format!(
            "{label} output exceeded 64 KiB"
        )));
    }
    if !status.success() {
        return Err(PortError::Unavailable(format!(
            "{label} exited {status}: {}",
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
        .map_err(|_| PortError::Unavailable("MetaHarness reader panicked".into()))?
        .map_err(|error| PortError::Unavailable(format!("MetaHarness output read failed: {error}")))
}

fn append_line(path: &Path, line: &str) -> Result<(), PortError> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| PortError::Unavailable(error.to_string()))?;
    writeln!(file, "{line}").map_err(|error| PortError::Unavailable(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn result(id: &str) -> CandidateResult {
        CandidateResult {
            candidate_id: id.into(),
            fitness: 0.7,
            scenario_digest: "a".repeat(64),
        }
    }

    #[test]
    fn submissions_append_as_json_lines() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("flywheel-feed.jsonl");
        let adapter = MetaharnessAdapter::new(&path);
        adapter.submit(&result("a")).unwrap();
        adapter.submit(&result("b")).unwrap();
        assert_eq!(fs::read_to_string(path).unwrap().lines().count(), 2);
    }

    #[test]
    fn missing_node_fails_closed() {
        let dir = tempdir().unwrap();
        let config = MetaharnessProcessConfig::new(
            dir.path().join("missing-node"),
            dir.path().join("missing-script"),
            dir.path().join("missing-darwin"),
            dir.path().join("missing-genome"),
            dir.path().join("missing-candidate"),
            dir.path(),
            dir.path().join("missing-entry"),
        );
        let runner = MetaharnessProcessRunner::new(config, dir.path().join("receipt.jsonl"));
        assert!(matches!(runner.evaluate(), Err(PortError::Unavailable(_))));
    }

    #[test]
    fn process_failure_modes_fail_closed() {
        let executable = std::env::current_exe().unwrap();
        let dir = tempdir().unwrap();

        let mut non_zero = Command::new(&executable);
        non_zero
            .args([
                "--ignored",
                "--exact",
                "tests::fixture_non_zero",
                "--nocapture",
            ])
            .current_dir(dir.path());
        assert!(matches!(
            run_command_bounded(non_zero, "fixture", Duration::from_secs(2)),
            Err(PortError::Unavailable(message)) if message.contains("exited")
        ));

        let mut timeout = Command::new(&executable);
        timeout
            .args([
                "--ignored",
                "--exact",
                "tests::fixture_timeout",
                "--nocapture",
            ])
            .current_dir(dir.path());
        assert!(matches!(
            run_command_bounded(timeout, "fixture", Duration::from_millis(30)),
            Err(PortError::Unavailable(message)) if message.contains("timed out")
        ));

        let mut oversized = Command::new(&executable);
        oversized
            .args([
                "--ignored",
                "--exact",
                "tests::fixture_oversized",
                "--nocapture",
            ])
            .current_dir(dir.path());
        assert!(matches!(
            run_command_bounded(oversized, "fixture", Duration::from_secs(2)),
            Err(PortError::Rejected(message)) if message.contains("64 KiB")
        ));
    }

    #[test]
    fn malformed_schema_and_receipt_failure_are_rejected() {
        assert!(serde_json::from_slice::<BridgeResponse>(br#"{"package":"wrong"}"#).is_err());
        let dir = tempdir().unwrap();
        assert!(matches!(
            append_line(dir.path(), "{}"),
            Err(PortError::Unavailable(_))
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
    fn passes_shared_flywheel_sink_contract() {
        let dir = tempdir().unwrap();
        let adapter = MetaharnessAdapter::new(dir.path().join("feed.jsonl"));
        jitter_contract_tests::assert_flywheel_sink(&adapter);
    }

    #[test]
    #[ignore = "requires Node, the pinned Flywheel package, and built jitter-candidate"]
    fn live_metaharness_round_trip_when_configured() {
        let node = std::env::var_os("JITTER_NODE_BIN").expect("JITTER_NODE_BIN");
        let script =
            std::env::var_os("JITTER_METAHARNESS_SCRIPT").expect("JITTER_METAHARNESS_SCRIPT");
        let darwin = std::env::var_os("JITTER_DARWIN_CLI").expect("JITTER_DARWIN_CLI");
        let genome = std::env::var_os("JITTER_DARWIN_GENOME").expect("JITTER_DARWIN_GENOME");
        let candidate = std::env::var_os("JITTER_CANDIDATE_BIN").expect("JITTER_CANDIDATE_BIN");
        let repository = std::env::var_os("JITTER_REPO").expect("JITTER_REPO");
        let entry =
            std::env::var_os("METAHARNESS_FLYWHEEL_ENTRY").expect("METAHARNESS_FLYWHEEL_ENTRY");
        let dir = tempdir().unwrap();
        let config = MetaharnessProcessConfig::new(
            node, script, darwin, genome, candidate, repository, entry,
        );
        let receipt = MetaharnessProcessRunner::new(config, dir.path().join("metaharness.jsonl"))
            .evaluate()
            .unwrap();
        assert_eq!(receipt.package, "@metaharness/flywheel@0.1.12");
        assert_eq!(receipt.darwin.package, "@metaharness/darwin@0.10.3");
        assert!(!receipt.darwin.winner.is_empty());
        assert_eq!(receipt.promotions, 0);
        assert!(receipt.replay_verdict.pass);
        assert_eq!(receipt.authority, "none");
    }
}
