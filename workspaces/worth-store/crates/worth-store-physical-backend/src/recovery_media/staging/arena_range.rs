use super::{
    CompletedRecoveryStagingWrite, IndeterminateRecoveryStagingWrite,
    RecoveryStagingIndeterminatePhysical, RecoveryStagingPhysicalFailure,
    RecoveryStagingWriteDisposition,
};
use crate::filesystem_media::{
    ArtifactRangeReadOutcome, ArtifactRangeWriteDurabilityRequirement, ArtifactRangeWriteOutcome,
    ArtifactTreeFile, ArtifactTreeMedia,
};
use sha2::{Digest, Sha256};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

pub(super) fn complete_existing(
    media: &ArtifactTreeMedia<'_>,
    artifact: RecordArtifactFile,
    physical: ArtifactTreeFile,
    coordinate: RecordFrameCoordinate,
    expected: &[u8],
) -> Result<CompletedRecoveryStagingWrite, RecoveryStagingPhysicalFailure> {
    let length = media
        .file_length(&physical)
        .map_err(RecoveryStagingPhysicalFailure::Denied)?;
    let end = coordinate.offset() + u64::from(coordinate.length());
    let mut prior_read = None;
    if end <= length {
        let mut observed = vec![0; expected.len()];
        match media.read_exact_range(&physical, coordinate, &mut observed) {
            ArtifactRangeReadOutcome::Completed(verified) if observed == expected => {
                return Ok(CompletedRecoveryStagingWrite {
                    artifact,
                    coordinate,
                    payload_digest: Sha256::digest(expected).into(),
                    disposition: RecoveryStagingWriteDisposition::AlreadyMaterialized,
                    created: None,
                    verified: Some(verified),
                    prefix_verified: None,
                    appended: None,
                    range_written: None,
                });
            }
            ArtifactRangeReadOutcome::Completed(read) => {
                prior_read = Some(read);
            }
            ArtifactRangeReadOutcome::DeniedBeforeEffect(failure) => {
                return Err(RecoveryStagingPhysicalFailure::Denied(failure))
            }
        }
    }
    match media.write_arena_range_exact_at(
        &physical,
        coordinate,
        expected,
        ArtifactRangeWriteDurabilityRequirement::BufferedWrite,
    ) {
        ArtifactRangeWriteOutcome::Completed(written) => Ok(CompletedRecoveryStagingWrite {
            artifact,
            coordinate,
            payload_digest: written.payload_digest(),
            disposition: RecoveryStagingWriteDisposition::RangeWritten,
            created: None,
            verified: prior_read,
            prefix_verified: None,
            appended: None,
            range_written: Some(written),
        }),
        ArtifactRangeWriteOutcome::DeniedBeforeEffect(failure) => {
            Err(RecoveryStagingPhysicalFailure::Denied(failure))
        }
        ArtifactRangeWriteOutcome::Indeterminate(written) => Err(
            RecoveryStagingPhysicalFailure::Indeterminate(IndeterminateRecoveryStagingWrite {
                artifact,
                payload_digest: Sha256::digest(expected).into(),
                physical: RecoveryStagingIndeterminatePhysical::Range(written),
            }),
        ),
    }
}
