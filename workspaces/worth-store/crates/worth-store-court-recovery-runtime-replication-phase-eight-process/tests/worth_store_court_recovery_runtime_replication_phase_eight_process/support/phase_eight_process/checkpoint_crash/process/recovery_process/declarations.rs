pub(crate) use {std::path::Path};
pub(crate) use {std::process::Command, std::process::Output, std::process::Stdio};
pub(crate) use {std::time::Duration};

pub(crate) use {worth_store_recovery_runtime::RecoveryReportEnvelope};

pub(crate) use {worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard, crate::support_binaries};

pub(crate) const CHECKPOINT_RECOVERY_PROFILE: &str = "c8-phase8-fate-coverage-v1";

pub(crate) fn fresh_recovery_raw(
    parent: &tempfile::TempDir,
    root: &Path,
    label: &str,
) -> (Output, Option<RecoveryReportEnvelope>) {
    let report_path = parent.path().join(format!("{label}-recovery-report.bin"));
    let child = worth_store_testkit_recovery_runtime::child_lifecycle::ProcessChildGuard::new(
        std::process::Command::new(
            crate::support_binaries::phase_eight_process_binaries()
                .recovery()
                .path(),
        )
        .arg(root)
        .arg(format!("--bounded-profile={CHECKPOINT_RECOVERY_PROFILE}"))
        .arg(format!("--report={}", report_path.display()))
        .env("TMP", parent.path())
        .env("TEMP", parent.path())
        .env("TMPDIR", parent.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("launch fresh checkpoint recovery"),
    );
    let output = child
        .wait_with_output_within(std::time::Duration::from_secs(120))
        .expect("wait for fresh checkpoint recovery");
    let report = std::fs::read(&report_path)
        .ok()
        .and_then(|encoded| worth_store_recovery_runtime::RecoveryReportEnvelope::decode(&encoded).ok());
    (output, report)
}