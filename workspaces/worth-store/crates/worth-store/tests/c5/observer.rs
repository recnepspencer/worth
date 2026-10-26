use std::path::Path;

pub(super) fn run(root: &Path) -> String {
    let binary = worth_store_test_support::OFFLINE_RECOVERY_OBSERVER.path();
    let output = std::process::Command::new(&binary)
        .arg("c5-current-manifest")
        .arg(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "offline observer {} failed\nstdout:\n{}\nstderr:\n{}",
        binary.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}
