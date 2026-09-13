use std::path::Path;

use worth_store_recovery_runtime::{PhysicalRecoveryOutcome, RecoveryReportEnvelope};

pub(super) fn persist(
    path: Option<&Path>,
    outcome: &PhysicalRecoveryOutcome,
) -> Result<(), String> {
    let Some(path) = path else {
        return Ok(());
    };
    std::fs::write(path, RecoveryReportEnvelope::from_outcome(outcome).encode())
        .map_err(|error| format!("could not write recovery report {path:?}: {error}"))
}
