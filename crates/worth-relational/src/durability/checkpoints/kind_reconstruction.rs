//! Checkpoint kind reconstruction distinguishes corrupt images from request stops.
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::storage::residency::RelationalResidencyDemandFailure;

pub(crate) fn stop(
    context: &str,
    failure: RelationalResidencyDemandFailure,
) -> RelationalRecoveryStop {
    match failure {
        RelationalResidencyDemandFailure::StorageUnavailable(failure) => DurabilityError::new(
            RecoveryFailureClass::CorruptCheckpoint,
            format!("{context}: {:?}", failure.cause),
        )
        .into(),
        other => other.into(),
    }
}
