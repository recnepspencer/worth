use sha2::{Digest, Sha256};
use worth_store_physical_format::{RecordArtifactFile, RecordFrameCoordinate};

use crate::filesystem_media::{
    ArtifactAppendOutcome, ArtifactAppendRange, ArtifactNewWriteOutcome, ArtifactNewWriteRange,
    ArtifactRangeReadOutcome, ArtifactTreeFile, ArtifactTreeMedia, CompletedArtifactRangeRead,
};

use super::{
    CompletedRecoveryStagingWrite, IndeterminateRecoveryStagingWrite,
    RecoveryStagingIndeterminatePhysical, RecoveryStagingPhysicalFailure,
    RecoveryStagingWriteDisposition,
};

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
    let expected_length = expected.len() as u64;
    if length > expected_length {
        return Err(RecoveryStagingPhysicalFailure::Denied(damaged()));
    }
    if length == expected_length {
        return verify_complete(media, artifact, physical, coordinate, expected);
    }

    let prefix_length =
        usize::try_from(length).map_err(|_| RecoveryStagingPhysicalFailure::Denied(damaged()))?;
    let prefix_verified = verify_prefix(media, artifact, &physical, prefix_length, expected)?;
    let suffix = &expected[prefix_length..];
    let range = ArtifactAppendRange::new(length, suffix.len() as u64)
        .ok_or_else(|| RecoveryStagingPhysicalFailure::Denied(damaged()))?;
    match media.append_artifact_exact_at(&physical, range, suffix) {
        ArtifactAppendOutcome::Completed(appended) => Ok(CompletedRecoveryStagingWrite {
            artifact,
            coordinate,
            payload_digest: Sha256::digest(expected).into(),
            disposition: RecoveryStagingWriteDisposition::CompletedFromExactPrefix,
            created: None,
            verified: None,
            prefix_verified,
            appended: Some(appended),
        }),
        ArtifactAppendOutcome::DeniedBeforeEffect(failure) => {
            Err(RecoveryStagingPhysicalFailure::Denied(failure))
        }
        ArtifactAppendOutcome::Indeterminate(append) => Err(
            RecoveryStagingPhysicalFailure::Indeterminate(IndeterminateRecoveryStagingWrite {
                artifact,
                payload_digest: Sha256::digest(expected).into(),
                physical: RecoveryStagingIndeterminatePhysical::Append {
                    prefix_verified,
                    append,
                },
            }),
        ),
    }
}

pub(super) fn create(
    media: &ArtifactTreeMedia<'_>,
    artifact: RecordArtifactFile,
    physical: ArtifactTreeFile,
    coordinate: RecordFrameCoordinate,
    bytes: &[u8],
) -> Result<CompletedRecoveryStagingWrite, RecoveryStagingPhysicalFailure> {
    let range = ArtifactNewWriteRange::new(bytes.len() as u64).expect("nonempty coordinate");
    match media.write_new_exact(&physical, range, bytes) {
        ArtifactNewWriteOutcome::Completed(created) => Ok(CompletedRecoveryStagingWrite {
            artifact,
            coordinate,
            payload_digest: created.payload_digest(),
            disposition: RecoveryStagingWriteDisposition::Created,
            created: Some(created),
            verified: None,
            prefix_verified: None,
            appended: None,
        }),
        ArtifactNewWriteOutcome::DeniedBeforeEffect(failure) => {
            Err(RecoveryStagingPhysicalFailure::Denied(failure))
        }
        ArtifactNewWriteOutcome::Indeterminate(physical) => Err(
            RecoveryStagingPhysicalFailure::Indeterminate(IndeterminateRecoveryStagingWrite {
                artifact,
                payload_digest: Sha256::digest(bytes).into(),
                physical: RecoveryStagingIndeterminatePhysical::NewArtifact(physical),
            }),
        ),
    }
}

fn verify_complete(
    media: &ArtifactTreeMedia<'_>,
    artifact: RecordArtifactFile,
    physical: ArtifactTreeFile,
    coordinate: RecordFrameCoordinate,
    expected: &[u8],
) -> Result<CompletedRecoveryStagingWrite, RecoveryStagingPhysicalFailure> {
    let mut observed = vec![0; expected.len()];
    let verified = read_exact(media, &physical, coordinate, &mut observed)?;
    if observed != expected {
        return Err(RecoveryStagingPhysicalFailure::Denied(damaged()));
    }
    Ok(CompletedRecoveryStagingWrite {
        artifact,
        coordinate,
        payload_digest: Sha256::digest(expected).into(),
        disposition: RecoveryStagingWriteDisposition::AlreadyMaterialized,
        created: None,
        verified: Some(verified),
        prefix_verified: None,
        appended: None,
    })
}

fn verify_prefix(
    media: &ArtifactTreeMedia<'_>,
    artifact: RecordArtifactFile,
    physical: &ArtifactTreeFile,
    prefix_length: usize,
    expected: &[u8],
) -> Result<Option<CompletedArtifactRangeRead>, RecoveryStagingPhysicalFailure> {
    if prefix_length == 0 {
        return Ok(None);
    }
    let length = u32::try_from(prefix_length)
        .map_err(|_| RecoveryStagingPhysicalFailure::Denied(damaged()))?;
    let coordinate = RecordFrameCoordinate::new(artifact, 0, length)
        .ok_or_else(|| RecoveryStagingPhysicalFailure::Denied(damaged()))?;
    let mut observed = vec![0; prefix_length];
    let verified = read_exact(media, physical, coordinate, &mut observed)?;
    if observed != expected[..prefix_length] {
        return Err(RecoveryStagingPhysicalFailure::Denied(damaged()));
    }
    Ok(Some(verified))
}

fn read_exact(
    media: &ArtifactTreeMedia<'_>,
    physical: &ArtifactTreeFile,
    coordinate: RecordFrameCoordinate,
    observed: &mut [u8],
) -> Result<CompletedArtifactRangeRead, RecoveryStagingPhysicalFailure> {
    match media.read_exact_range(physical, coordinate, observed) {
        ArtifactRangeReadOutcome::Completed(completed) => Ok(completed),
        ArtifactRangeReadOutcome::DeniedBeforeEffect(failure) => {
            Err(RecoveryStagingPhysicalFailure::Denied(failure))
        }
    }
}

fn damaged() -> crate::filesystem_media::ArtifactTreeFailure {
    crate::filesystem_media::ArtifactTreeFailure::recovery_damaged()
}
