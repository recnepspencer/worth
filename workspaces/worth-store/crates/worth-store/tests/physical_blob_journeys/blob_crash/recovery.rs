use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub(crate) fn recover_closed_store(root: &Path) {
    recover_closed_store_with_profile(root, "c11-blob-crash-v1");
}

pub(crate) fn recover_closed_store_with_profile(root: &Path, profile: &str) {
    let binary = std::env::var_os("WORTH_C8_RECOVERY_EXECUTABLE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap()
                .parent()
                .and_then(Path::parent)
                .unwrap()
                .join(format!(
                    "physical_store_recover{}",
                    std::env::consts::EXE_SUFFIX
                ))
        });
    assert!(
        binary.is_file(),
        "standalone C8 recovery missing: {}; prebuild -p worth-store-recovery-runtime --bin physical_store_recover",
        binary.display()
    );
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
