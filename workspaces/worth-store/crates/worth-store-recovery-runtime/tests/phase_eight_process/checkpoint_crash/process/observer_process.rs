use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::Duration;

use worth_store_offline_verifier::RecoveryObserverReport;

use super::super::super::{child_lifecycle::ProcessChildGuard, support_binaries};

pub(crate) fn fresh_observer_raw(
    parent: &tempfile::TempDir,
    root: &Path,
    label: &str,
) -> (Output, Option<RecoveryObserverReport>) {
    let report_path = parent.path().join(format!("{label}-observer-report.bin"));
    let child = ProcessChildGuard::new(
        Command::new(
            support_binaries::phase_eight_process_binaries()
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
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch fresh checkpoint observer"),
    );
    let output = child
        .wait_with_output_within(Duration::from_secs(120))
        .expect("wait for fresh checkpoint observer");
    let report = std::fs::read(&report_path)
        .ok()
        .and_then(|encoded| RecoveryObserverReport::decode(&encoded).ok());
    (output, report)
}
