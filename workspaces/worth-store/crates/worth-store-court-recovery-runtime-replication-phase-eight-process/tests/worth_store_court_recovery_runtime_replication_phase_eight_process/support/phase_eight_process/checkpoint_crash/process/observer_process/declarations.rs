pub(crate) use {std::path::Path};
pub(crate) use {std::process::Command, std::process::Output, std::process::Stdio};
pub(crate) use {std::time::Duration};

pub(crate) use {worth_store_offline_verifier::RecoveryObserverReport};

pub(crate) use {worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard, crate::support_binaries};

pub(crate) fn fresh_observer_raw(
    parent: &tempfile::TempDir,
    root: &Path,
    label: &str,
) -> (Output, Option<RecoveryObserverReport>) {
    let report_path = parent.path().join(format!("{label}-observer-report.bin"));
    let child = worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard::new(
        std::process::Command::new(
            crate::support_binaries::phase_eight_process_binaries()
                .observer()
                .path(),
        )
        .arg("c8-recovery-observe")
        .arg(root)
        .arg(&report_path)
        .args(["32768", "16384", "16384", "536870912"])
        .env("TMP", parent.path())
        .env("TEMP", parent.path())
        .env("TMPDIR", parent.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch fresh checkpoint observer"),
    );
    let output = child
        .wait_with_output_within(std::time::Duration::from_secs(120))
        .expect("wait for fresh checkpoint observer");
    let report = std::fs::read(&report_path)
        .ok()
        .and_then(|encoded| worth_store_offline_verifier::RecoveryObserverReport::decode(&encoded).ok());
    (output, report)
}