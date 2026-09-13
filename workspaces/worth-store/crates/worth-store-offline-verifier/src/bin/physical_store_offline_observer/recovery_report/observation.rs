use std::path::Path;

use worth_store_offline_verifier::{
    observe_recovery_artifacts, RecoveryObserverLimits, RecoveryObserverReport,
};

pub(super) fn execute(
    root: &Path,
    limits: RecoveryObserverLimits,
) -> Result<RecoveryObserverReport, String> {
    observe_recovery_artifacts(root, limits)
        .map_err(|denial| format!("recovery observation denied: {denial:?}"))
}
