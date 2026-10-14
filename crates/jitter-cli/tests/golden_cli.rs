//! Golden CLI test (docs/architecture.md §5, row 20's acceptance criterion):
//! `jitter-map run fixtures/fleet-retry-storm-22.json` must exit 0 and its stdout
//! `Receipt` JSON must match a committed golden fixture byte-for-byte.
//!
//! The `Receipt` printed to stdout depends only on the scenario, seed, and admission
//! strategy — never on prior memory-sink state (`nearest_prior_ids` is returned
//! alongside the receipt but is not part of what the CLI prints), so running this
//! test against a fresh temp memory directory is deterministic regardless of
//! anything another test run may have persisted elsewhere.

use std::process::Command;

#[test]
fn fleet_retry_storm_22_weighted_matches_committed_golden_output() {
    let memory_dir = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jitter-map"))
        .args([
            "--scenario",
            "../../fixtures/fleet-retry-storm-22.json",
            "--policy",
            "weighted-tenant-fair-budget",
            "--memory",
        ])
        .arg(memory_dir.path())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("jitter-map binary must run");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("stdout must be valid UTF-8");
    let golden = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/golden/fleet-retry-storm-22.weighted.json"
    ))
    .expect("golden fixture must exist");

    assert_eq!(actual.trim_end(), golden.trim_end());
}
