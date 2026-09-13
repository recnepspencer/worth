use worth_store::physical_runtime::{ObservedRecoveryArtifact, RecoveryDiscoveryFailure};
use worth_store_physical_format::RecordArtifactFile;
use worth_store_recovery_physics::PhysicalRedoTargetIdentity;

use super::super::PageObservationFailure;

pub(super) fn required_observed(
    result: Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure>,
    target: Option<PhysicalRedoTargetIdentity>,
    artifact: RecordArtifactFile,
) -> Result<ObservedRecoveryArtifact, PageObservationFailure> {
    match result {
        Ok(observed) if observed.bytes().is_some() => Ok(observed),
        Ok(_) => Err(PageObservationFailure::MissingArtifact { target, artifact }),
        Err(RecoveryDiscoveryFailure::ByteLimitExceeded { .. }) => {
            Err(PageObservationFailure::ByteLimit)
        }
        Err(failure) => Err(PageObservationFailure::Media { target, failure }),
    }
}
