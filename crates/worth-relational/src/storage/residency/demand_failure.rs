use crate::storage::data::RelationalStorageUnavailable;

/// A demand can stop under the original request without damaging its cell.
/// Storage refusals are sticky; request cancellation and budgets are retryable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RelationalResidencyDemandFailure {
    StorageUnavailable(RelationalStorageUnavailable),
    Request(crate::execution::RelationalExecutionDenialCause),
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
        }
    }
}

impl From<RelationalResidencyDemandFailure> for crate::durability::data::RelationalRecoveryStop {
    fn from(failure: RelationalResidencyDemandFailure) -> Self {
        match failure {
            RelationalResidencyDemandFailure::StorageUnavailable(failure) => failure.into(),
            RelationalResidencyDemandFailure::Request(cause) => Self::Request(cause),
        }
    }
}
