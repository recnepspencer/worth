pub(crate) use std::path::Path;

pub(crate) use worth_store_offline_verifier::RecoveryObserverReport;

pub(crate) use crate::process::support::checkpoint_crash::observer_process::fresh_observer_raw;

pub(crate) fn fresh_observer(
    parent: &tempfile::TempDir,
    root: &Path,
    label: &str,
) -> RecoveryObserverReport {
    let (output, report) = fresh_observer_raw(parent, root, label);
    assert!(
        output.status.success(),
        "checkpoint observer failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    report.expect("checkpoint observer report missing")
}
