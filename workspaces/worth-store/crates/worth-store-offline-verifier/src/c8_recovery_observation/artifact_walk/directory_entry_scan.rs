use std::path::Path;

use super::super::super::{
    RecoveryObserverCounters, RecoveryObserverLimits, RecoveryObserverObservationDenial,
    RecoveryObserverObservationFailure,
};

pub(super) fn admitted(
    directory: &Path,
    limits: RecoveryObserverLimits,
    counters: &mut RecoveryObserverCounters,
) -> Result<Vec<std::fs::DirEntry>, RecoveryObserverObservationFailure> {
    let entries = std::fs::read_dir(directory).map_err(|error| {
        RecoveryObserverObservationFailure::at_path(
            RecoveryObserverObservationDenial::Media(error.kind()),
            *counters,
            directory,
        )
    })?;
    counters.record_directory_opened().ok_or_else(|| {
        RecoveryObserverObservationFailure::at_path(
            RecoveryObserverObservationDenial::ArtifactChanged,
            *counters,
            directory,
        )
    })?;
    let remaining = limits
        .maximum_directory_entries()
        .saturating_sub(counters.directory_entries_observed());
    let capacity = usize::try_from(remaining.min(256)).unwrap_or(256);
    let mut admitted = Vec::with_capacity(capacity);
    for entry in entries {
        let entry = entry.map_err(|error| {
            RecoveryObserverObservationFailure::at_path(
                RecoveryObserverObservationDenial::Media(error.kind()),
                *counters,
                directory,
            )
        })?;
        let observed = counters.record_directory_entry().ok_or_else(|| {
            RecoveryObserverObservationFailure::at_path(
                RecoveryObserverObservationDenial::ArtifactChanged,
                *counters,
                directory,
            )
        })?;
        if observed > limits.maximum_directory_entries() {
            return Err(RecoveryObserverObservationFailure::at_path(
                RecoveryObserverObservationDenial::DirectoryEntryLimit {
                    observed,
                    admitted: limits.maximum_directory_entries(),
                },
                *counters,
                &entry.path(),
            ));
        }
        admitted.push(entry);
    }
    Ok(admitted)
}
