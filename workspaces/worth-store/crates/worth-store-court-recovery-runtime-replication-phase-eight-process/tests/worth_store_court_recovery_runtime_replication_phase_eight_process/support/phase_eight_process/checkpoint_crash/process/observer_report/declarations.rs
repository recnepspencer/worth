pub(crate) use {std::path::Path};

pub(crate) use {worth_store_offline_verifier::RecoveryObserverReport};

pub(crate) use {crate::phase_eight_process::checkpoint_crash::process::observer_process::fresh_observer_raw};

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