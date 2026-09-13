use std::path::Path;

use super::super::{
    RecoveryObserverCounters, RecoveryObserverObservationDenial, RecoveryObserverObservationFailure,
};

pub(super) fn relative_artifact_path(
    root: &Path,
    path: &Path,
    counters: RecoveryObserverCounters,
) -> Result<Box<str>, RecoveryObserverObservationFailure> {
    let relative = path.strip_prefix(root).map_err(|_| {
        RecoveryObserverObservationFailure::at_path(
            RecoveryObserverObservationDenial::ArtifactChanged,
            counters,
            path,
        )
    })?;
    let relative = relative.to_str().ok_or_else(|| {
        RecoveryObserverObservationFailure::at_path(
            RecoveryObserverObservationDenial::NonUnicodePath,
            counters,
            path,
        )
    })?;
    Ok(relative.replace('\\', "/").into_boxed_str())
}
