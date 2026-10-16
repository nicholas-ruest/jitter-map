use jitter_ports::PortError;
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub(crate) const MAX_OUTPUT_BYTES: usize = 64 * 1024;

pub(crate) fn canonical_file(path: &Path) -> Result<PathBuf, PortError> {
    let canonical = path
        .canonicalize()
        .map_err(|error| PortError::Unavailable(format!("executable unavailable: {error}")))?;
    if !canonical.is_file() {
        return Err(PortError::Unavailable("executable is not a file".into()));
    }
    Ok(canonical)
}

pub(crate) fn canonical_dir(path: &Path) -> Result<PathBuf, PortError> {
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

pub(crate) fn run_bounded(
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

pub(crate) fn append_receipt<T: Serialize>(path: &Path, receipt: &T) -> Result<(), PortError> {
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
