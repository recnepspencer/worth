pub(crate) fn run_with_private_authority(test_name: &str) -> bool {
    const CHILD_TEST: &str = "WORTH_SIGNAL_PRIVATE_PLACEMENT_AUTHORITY";
    if std::env::var(CHILD_TEST).as_deref() == Ok(test_name) {
        return false;
    }
    let output = std::process::Command::new(
        std::env::current_exe().expect("current test binary should be discoverable"),
    )
    .args(["--exact", test_name, "--nocapture"])
    .env(CHILD_TEST, test_name)
    .output()
    .expect("private authority test child should start");
    assert!(
        output.status.success(),
        "private authority test {test_name} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let receipt = stdout
        .lines()
        .find(|line| line.starts_with("PLACEMENT_RENDEZVOUS"))
        .expect("private authority child must execute the named test and record its rendezvous");
    println!("PRIVATE_AUTHORITY {test_name}: {receipt}");
    true
}
