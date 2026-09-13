pub(super) fn run_in_isolated_counter_process(test_name: &str, child_env: &str) -> bool {
    if std::env::var_os(child_env).is_some() {
        return false;
    }

    let output = std::process::Command::new(
        std::env::current_exe().expect("current test binary should be discoverable"),
    )
    .args(["--exact", test_name, "--nocapture"])
    .env(child_env, "1")
    .output()
    .expect("isolated counter test child should start");
    assert!(
        output.status.success(),
        "isolated counter test {test_name} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    true
}
