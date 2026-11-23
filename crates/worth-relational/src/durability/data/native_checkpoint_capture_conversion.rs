//! Translate framed recovery and residency stops at the checkpoint boundary.
use super::{DurabilityError, RecoveryFailureClass, RelationalNativeCheckpointCaptureDenial};

impl
    From<
        crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial,
    > for RelationalNativeCheckpointCaptureDenial
{
    fn from(
        error: crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial,
    ) -> Self {
        use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifactEncodeDenial as Encode;
        match error {
            Encode::Frame(cause) => Self::ArtifactFrame(cause),
            Encode::Request(cause) => Self::Request(cause),
            Encode::Allocation(error) => Self::Allocation(error),
            Encode::Readmission(cause) => {
                crate::durability::checkpoint_history::capture::capture_stop(cause)
            }
            Encode::StorageUnavailable(failure) => Self::Durability(failure.into()),
            error => Self::Durability(DurabilityError::new(
                RecoveryFailureClass::CorruptCheckpoint,
                format!("inline canonical artifact encoding: {error}"),
            )),
        }
    }
}

impl From<crate::storage::residency::RelationalResidencyDemandFailure>
    for RelationalNativeCheckpointCaptureDenial
{
    fn from(failure: crate::storage::residency::RelationalResidencyDemandFailure) -> Self {
        use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
        match failure {
            Failure::Content(cause) => cause.into(),
            Failure::StorageUnavailable(failure) => failure.into(),
            Failure::Request(cause) => Self::Request(cause),
        }
    }
}
