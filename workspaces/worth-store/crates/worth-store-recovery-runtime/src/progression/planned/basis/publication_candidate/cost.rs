use worth_store_physical_format::{DurablePhysicalRootManifest, RecordArtifactFile};

use super::{CandidateBuildDenial, RecoveryPublicationCandidateArtifact};

pub(super) fn candidate_materialization_bytes(
    _root: &DurablePhysicalRootManifest,
    referenced_artifacts: &[RecordArtifactFile],
    artifacts: &[RecoveryPublicationCandidateArtifact],
) -> Result<u64, CandidateBuildDenial> {
    let root_bytes = std::mem::size_of::<DurablePhysicalRootManifest>() as u64;
    let reference_bytes = (referenced_artifacts.len() as u64)
        .checked_mul(std::mem::size_of::<RecordArtifactFile>() as u64)
        .ok_or(CandidateBuildDenial::Invalid)?;
    let descriptor_bytes = (artifacts.len() as u64)
        .checked_mul(std::mem::size_of::<RecoveryPublicationCandidateArtifact>() as u64)
        .ok_or(CandidateBuildDenial::Invalid)?;
    artifacts.iter().try_fold(
        root_bytes
            .checked_add(reference_bytes)
            .and_then(|bytes| bytes.checked_add(descriptor_bytes))
            .ok_or(CandidateBuildDenial::Invalid)?,
        |bytes, artifact| {
            bytes
                .checked_add(artifact.bytes.len() as u64)
                .ok_or(CandidateBuildDenial::Invalid)
        },
    )
}
