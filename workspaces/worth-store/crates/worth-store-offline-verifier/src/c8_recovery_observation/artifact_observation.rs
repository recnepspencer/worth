use std::path::{Path, PathBuf};

use super::artifact_observation::bounded_artifact_read::read_bounded_artifact;
use super::artifact_observation::relative_artifact_path::relative_artifact_path;
use super::artifact_walk::ObservedRecoveryArtifact;
use super::physical_format;
use super::{RecoveryObserverCounters, RecoveryObserverLimits, RecoveryObserverObservationFailure};

#[path = "artifact_observation/bounded_artifact_read.rs"]
mod bounded_artifact_read;
#[path = "artifact_observation/relative_artifact_path.rs"]
mod relative_artifact_path;

pub(super) fn observe_file(
    root: &Path,
    path: PathBuf,
    limits: RecoveryObserverLimits,
    counters: &mut RecoveryObserverCounters,
) -> Result<ObservedRecoveryArtifact, RecoveryObserverObservationFailure> {
    let read = read_bounded_artifact(&path, limits, counters)?;
    let relative = relative_artifact_path(root, &path, *counters)?;
    counters.record_artifact_observed().ok_or_else(|| {
        super::RecoveryObserverObservationFailure::at_path(
            super::RecoveryObserverObservationDenial::ArtifactChanged,
            *counters,
            &path,
        )
    })?;
    let evidence = physical_format::observe(&relative, &read.contents);
    Ok(ObservedRecoveryArtifact {
        path: relative,
        byte_length: read.byte_length,
        digest: read.digest,
        evidence,
    })
}
