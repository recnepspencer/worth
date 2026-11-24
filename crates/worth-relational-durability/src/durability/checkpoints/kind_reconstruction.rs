//! Checkpoint kind reconstruction distinguishes corrupt images from request stops.
use worth_relational_contracts::internal_port::durability::data::recovery_errors::{
    DurabilityError, RecoveryFailureClass,
};
use worth_relational_contracts::internal_port::durability::data::recovery_stop::RelationalRecoveryStop;
use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure;

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
