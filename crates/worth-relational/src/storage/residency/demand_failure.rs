use crate::storage::data::RelationalStorageUnavailable;

/// A demand can stop under the original request without damaging its cell.
/// Proven member damage is sticky; request and transient refusals are retryable.
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

impl From<RelationalResidencyDemandFailure> for crate::durability::data::RelationalRecoveryStop {
    fn from(failure: RelationalResidencyDemandFailure) -> Self {
        match failure {
            RelationalResidencyDemandFailure::StorageUnavailable(failure) => failure.into(),
            RelationalResidencyDemandFailure::Request(cause) => Self::Request(cause),
        }
    }
}
