use crate::storage::data::RelationalStorageUnavailable;

/// A demand can stop under the original request without damaging its cell.
/// Proven member damage is sticky; request and transient refusals are retryable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalResidencyDemandFailure {
    StorageUnavailable(RelationalStorageUnavailable),
    Request(crate::execution::RelationalExecutionDenialCause),
    Content(crate::content::RelationalContentDenial),
}

impl From<RelationalStorageUnavailable> for RelationalResidencyDemandFailure {
    fn from(failure: RelationalStorageUnavailable) -> Self {
        Self::StorageUnavailable(failure)
    }
}

impl From<RelationalResidencyDemandFailure> for crate::execution::RelationalExecutionDenialCause {
    fn from(failure: RelationalResidencyDemandFailure) -> Self {
        match failure {
            RelationalResidencyDemandFailure::StorageUnavailable(failure) => {
                Self::StorageUnavailable(failure)
            }
            RelationalResidencyDemandFailure::Request(cause) => cause,
            RelationalResidencyDemandFailure::Content(cause) => cause.into_execution_cause(),
        }
    }
}

impl From<RelationalResidencyDemandFailure>
    for crate::durability::data::recovery_stop::RelationalRecoveryStop
{
    fn from(failure: RelationalResidencyDemandFailure) -> Self {
        match failure {
            RelationalResidencyDemandFailure::StorageUnavailable(failure) => failure.into(),
            RelationalResidencyDemandFailure::Request(cause) => Self::Request(cause),
            RelationalResidencyDemandFailure::Content(cause) => cause.into(),
        }
    }
}

impl From<RelationalResidencyDemandFailure>
    for crate::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial
{
    fn from(failure: RelationalResidencyDemandFailure) -> Self {
        use RelationalResidencyDemandFailure as Failure;
        match failure {
            Failure::Content(cause) => cause.into(),
            Failure::StorageUnavailable(failure) => failure.into(),
            Failure::Request(cause) => Self::Request(cause),
        }
    }
}
