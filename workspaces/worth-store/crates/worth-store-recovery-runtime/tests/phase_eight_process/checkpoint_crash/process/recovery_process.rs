use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use worth_store_recovery_runtime::RecoveryReportEnvelope;

use super::super::super::{child_lifecycle::ProcessChildGuard, support_binaries};

const CHECKPOINT_RECOVERY_PROFILE: &str = "c8-phase8-fate-coverage-v1";

pub(crate) fn fresh_recovery_raw(
    parent: &tempfile::TempDir,
    root: &Path,
    label: &str,
) -> (Output, Option<RecoveryReportEnvelope>) {
    let report_path = parent.path().join(format!("{label}-recovery-report.bin"));
    let child = ProcessChildGuard::new(
        Command::new(
            support_binaries::phase_eight_process_binaries()
                .recovery()
                .path(),
        )
        .arg(root)
        .arg(format!("--bounded-profile={CHECKPOINT_RECOVERY_PROFILE}"))
        .arg(format!("--report={}", report_path.display()))
        .env("TMP", parent.path())
        .env("TEMP", parent.path())
        .env("TMPDIR", parent.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch fresh checkpoint recovery"),
    );
    let output = child
        .wait_with_output_within(Duration::from_secs(120))
        .expect("wait for fresh checkpoint recovery");
    let report = std::fs::read(&report_path)
        .ok()
        .and_then(|encoded| RecoveryReportEnvelope::decode(&encoded).ok());
    (output, report)
}
