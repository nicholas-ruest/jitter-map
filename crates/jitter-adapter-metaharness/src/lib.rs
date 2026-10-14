//! Infrastructure Adapter: `MetaharnessAdapter` implements `FlywheelSink`, appending
//! every submitted candidate result to an append-only JSON-Lines feed that the
//! Darwin/Flywheel replay loop reads as its evaluation feedback stream.
//!
//! ADR-0115 (Evaluation Feedback Boundary; extends ADR-0023); docs/ddd/09-upstream-integrations.md.
//!
//! **Provenance note (docs/integration-matrix.md, MetaHarness row)**: the inspected
//! source commit for Darwin/Flywheel has no corresponding npm `gitHead`, so this
//! adapter's output is evidence a human or Flywheel process reads; it is not itself
//! the thing being pinned. This matches ADR-0014 (append-only evidence) — results
//! are appended, never rewritten in place.

use jitter_ports::{CandidateResult, FlywheelSink, PortError};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

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
        let line = serde_json::to_string(result).map_err(|e| PortError::Rejected(e.to_string()))?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.feed_path)
            .map_err(|e| PortError::Unavailable(e.to_string()))?;
        writeln!(file, "{line}").map_err(|e| PortError::Unavailable(e.to_string()))?;
        Ok(())
    }
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
        let contents = fs::read_to_string(&path).unwrap();
        assert_eq!(contents.lines().count(), 2);
    }

    #[test]
    fn existing_lines_are_never_rewritten() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("flywheel-feed.jsonl");
        let adapter = MetaharnessAdapter::new(&path);
        adapter.submit(&result("a")).unwrap();
        let first_line = fs::read_to_string(&path).unwrap();
        adapter.submit(&result("b")).unwrap();
        let after = fs::read_to_string(&path).unwrap();
        assert!(after.starts_with(first_line.trim_end()));
    }

    #[test]
    fn passes_shared_flywheel_sink_contract() {
        let dir = tempdir().unwrap();
        let adapter = MetaharnessAdapter::new(dir.path().join("feed.jsonl"));
        jitter_contract_tests::assert_flywheel_sink(&adapter);
    }
}
