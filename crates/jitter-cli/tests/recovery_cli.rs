//! Automates the manual verification recorded in ADR-0120 and docs/architecture.md §7:
//! a plain run, an `--interrupt-after 6` capture, and a `--replay` of that checkpoint
//! must all produce the identical `Receipt` (digest for digest) on
//! `fixtures/fleet-retry-storm-22.json`.

use std::process::Command;

fn run_jitter_map(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_jitter-map"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("jitter-map binary must run")
}

#[test]
fn interrupted_and_replayed_run_matches_uninterrupted_receipt() {
    let baseline_memory = tempfile::tempdir().unwrap();
    let baseline = run_jitter_map(&[
        "--scenario",
        "../../fixtures/fleet-retry-storm-22.json",
        "--policy",
        "weighted-tenant-fair-budget",
        "--memory",
        baseline_memory.path().to_str().unwrap(),
    ]);
    assert!(
        baseline.status.success(),
        "{}",
        String::from_utf8_lossy(&baseline.stderr)
    );

    let interrupted_memory = tempfile::tempdir().unwrap();
    let checkpoint_path = interrupted_memory.path().join("checkpoint.json");
    let capture = run_jitter_map(&[
        "--scenario",
        "../../fixtures/fleet-retry-storm-22.json",
        "--policy",
        "weighted-tenant-fair-budget",
        "--memory",
        interrupted_memory.path().to_str().unwrap(),
        "--interrupt-after",
        "6",
        "--checkpoint-out",
        checkpoint_path.to_str().unwrap(),
    ]);
    assert!(
        capture.status.success(),
        "{}",
        String::from_utf8_lossy(&capture.stderr)
    );
    assert!(checkpoint_path.exists());

    let replay = run_jitter_map(&[
        "--scenario",
        "../../fixtures/fleet-retry-storm-22.json",
        "--policy",
        "weighted-tenant-fair-budget",
        "--memory",
        interrupted_memory.path().to_str().unwrap(),
        "--replay",
        checkpoint_path.to_str().unwrap(),
    ]);
    assert!(
        replay.status.success(),
        "{}",
        String::from_utf8_lossy(&replay.stderr)
    );

    let baseline_stdout = String::from_utf8(baseline.stdout).unwrap();
    let replay_stdout = String::from_utf8(replay.stdout).unwrap();
    assert_eq!(baseline_stdout, replay_stdout);
}
