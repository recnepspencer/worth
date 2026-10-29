//! Re-execute process-global courts without giving each one a linked binary.
pub fn reexecute_isolated_test() -> bool {
    let thread = std::thread::current();
    let name = thread
        .name()
        .expect("libtest names the running test thread");
    const CHILD: &str = "WORTH_INTEGRATION_ISOLATED_TEST";
    if std::env::var(CHILD).as_deref() == Ok(name) {
        return false;
    }
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", name, "--nocapture"])
        .env(CHILD, name)
        .output()
        .expect("start the exact court in a fresh process");
    assert!(
        output.status.success(),
        "isolated court {name} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"),
        "the exact isolated court must execute: {name}"
    );
    true
}