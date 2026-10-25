use std::{path::Path, process::Command};

pub(crate) fn recover_closed_store(root: &Path) {
    recover_closed_store_with_profile(root, "c11-blob-crash-v1");
}

pub(crate) fn recover_closed_store_with_profile(root: &Path, profile: &str) {
    let binary = worth_store_test_support::FRESH_PROCESS_RECOVERY.path();
    let output = Command::new(binary)
        .arg(root)
        .arg(format!("--bounded-profile={profile}"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "fresh C8 recovery failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("C8_RECOVERY_RUNTIME "),
        "fresh C8 process did not report completed recovery: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
