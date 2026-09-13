use std::process::Command;

#[test]
fn global_help_succeeds_on_stdout() {
    let output = Command::new(env!("CARGO_BIN_EXE_store-test-runner"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)
        .unwrap()
        .starts_with("usage: store-test-runner"));
    assert!(output.stderr.is_empty());
}

#[test]
fn unknown_command_fails_on_stderr() {
    let output = Command::new(env!("CARGO_BIN_EXE_store-test-runner"))
        .arg("definitely-not-a-command")
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("unknown command `definitely-not-a-command`"));
    assert!(stderr.contains("usage: store-test-runner"));
}

#[test]
fn process_scenario_rejects_sharding_before_execution() {
    let output = Command::new(env!("CARGO_BIN_EXE_store-test-runner"))
        .args([
            "ci",
            "--partition",
            "process-scenario",
            "--shard-index",
            "0",
            "--shard-count",
            "2",
            "--list",
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("process-scenario partition is not shardable"));
}

#[test]
fn list_displays_the_direct_plan_without_running_tests() {
    let output = Command::new(env!("CARGO_BIN_EXE_store-test-runner"))
        .args(["smoke", "--list"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("smoke::worth-store"));
    assert!(stdout.contains("cargo nextest run"));
    assert!(!stdout.contains("run: "));
    assert!(output.stderr.is_empty());
}
