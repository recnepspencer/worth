use std::path::Path;

use worth_store_offline_verifier::RecoveryObserverReport;

pub(super) fn write(path: &Path, report: &RecoveryObserverReport) -> Result<(), String> {
    std::fs::write(path, report.encode())
        .map_err(|error| format!("could not write recovery observer report: {error}"))
}
