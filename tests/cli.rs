use std::process::Command;

#[test]
fn json_output_is_reproducible_and_machine_readable() {
    let args = [
        "--clients",
        "2",
        "--attempts",
        "3",
        "--seed",
        "99",
        "--format",
        "json",
    ];
    let first = Command::new(env!("CARGO_BIN_EXE_jitter-map"))
        .args(args)
        .output()
        .unwrap();
    let second = Command::new(env!("CARGO_BIN_EXE_jitter-map"))
        .args(args)
        .output()
        .unwrap();

    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    let value: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(value["summary"]["total_events"], 6);
    assert_eq!(value["events"].as_array().unwrap().len(), 6);
}

#[test]
fn invalid_configuration_fails_with_actionable_message() {
    let output = Command::new(env!("CARGO_BIN_EXE_jitter-map"))
        .args(["--base", "10s", "--cap", "1s"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("base delay cannot exceed the cap"));
}

#[test]
fn csv_summary_has_a_stable_schema() {
    let output = Command::new(env!("CARGO_BIN_EXE_jitter-map"))
        .args(["--summary-only", "--format", "csv"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("metric,value\n"));
    assert!(stdout.contains("collision_pairs,"));
    assert!(stdout.contains("peak_window_load,"));
}
