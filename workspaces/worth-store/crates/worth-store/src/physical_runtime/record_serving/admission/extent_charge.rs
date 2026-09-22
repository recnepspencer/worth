use worth_store_physical_backend::ArtifactTreeFailureKind;
use worth_store_physical_format::RecordArtifactFile;

use super::super::residency::serving_artifacts::ServingRecordArtifacts;
use super::bootstrap::{
    backend_before_effect, BootstrapTransitionFailure, RecordBootstrapDenial,
};

/// Bytes retained by published extent artifacts.
///
/// The first publication of an extent id uses generation 1. A frontier gap is
/// an id that was reserved and then denied before either file existed. A
/// published extent contributes its data file and manifest.
pub(super) fn retained_extent_bytes(
    artifacts: &ServingRecordArtifacts,
    next_extent: u64,
) -> Result<u64, BootstrapTransitionFailure> {
    let mut retained = 0_u64;
    let mut extent = 1_u64;
    while extent < next_extent {
        let data = optional_length(
            artifacts,
            RecordArtifactFile::Extent {
                extent,
                generation: 1,
            },
        )?;
        let manifest = optional_length(
            artifacts,
            RecordArtifactFile::ExtentManifest {
                extent,
                generation: 1,
            },
        )?;
        retained = retained.saturating_add(match (data, manifest) {
            (Some(data), Some(manifest)) => data.saturating_add(manifest),
            (None, None) => 0,
            _ => {
                return Err(BootstrapTransitionFailure::Denied(
                    RecordBootstrapDenial::CurrentRootDamaged,
                ))
            }
        });
        extent = extent.saturating_add(1);
    }
    Ok(retained)
}

fn optional_length(
    artifacts: &ServingRecordArtifacts,
    artifact: RecordArtifactFile,
) -> Result<Option<u64>, BootstrapTransitionFailure> {
    match artifacts.file_length(artifact) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(failure) if failure.kind() == ArtifactTreeFailureKind::Absent => Ok(None),
        Err(failure) => Err(backend_before_effect(failure)),
    }
}
