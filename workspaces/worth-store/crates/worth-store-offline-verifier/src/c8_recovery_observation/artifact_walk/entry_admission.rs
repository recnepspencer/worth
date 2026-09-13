use super::super::{
    RecoveryObserverCounters, RecoveryObserverLimits, RecoveryObserverObservationDenial,
    RecoveryObserverObservationFailure,
};
use super::entry_classification::ClassifiedEntry;

pub(super) enum AdmittedEntry {
    Directory(std::path::PathBuf),
    Artifact(std::path::PathBuf),
    IgnoredLock,
}

pub(super) fn admit(
    entry: ClassifiedEntry,
    limits: RecoveryObserverLimits,
    counters: &mut RecoveryObserverCounters,
) -> Result<AdmittedEntry, RecoveryObserverObservationFailure> {
    match entry {
        ClassifiedEntry::Directory(path) => {
            let observed = counters.record_directory_admitted().ok_or_else(|| {
                RecoveryObserverObservationFailure::at_path(
                    RecoveryObserverObservationDenial::ArtifactChanged,
                    *counters,
                    &path,
                )
            })?;
            if observed > limits.maximum_directories() {
                return Err(RecoveryObserverObservationFailure::at_path(
                    RecoveryObserverObservationDenial::DirectoryLimit {
                        observed,
                        admitted: limits.maximum_directories(),
                    },
                    *counters,
                    &path,
                ));
            }
            Ok(AdmittedEntry::Directory(path))
        }
        ClassifiedEntry::Artifact(path) => {
            let observed = counters.record_artifact_admitted().ok_or_else(|| {
                RecoveryObserverObservationFailure::at_path(
                    RecoveryObserverObservationDenial::ArtifactChanged,
                    *counters,
                    &path,
                )
            })?;
            if observed > limits.maximum_artifacts() {
                return Err(RecoveryObserverObservationFailure::at_path(
                    RecoveryObserverObservationDenial::ArtifactLimit {
                        observed,
                        admitted: limits.maximum_artifacts(),
                    },
                    *counters,
                    &path,
                ));
            }
            Ok(AdmittedEntry::Artifact(path))
        }
        ClassifiedEntry::IgnoredLock => Ok(AdmittedEntry::IgnoredLock),
    }
}
