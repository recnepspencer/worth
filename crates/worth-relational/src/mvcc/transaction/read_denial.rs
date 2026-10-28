//! A transaction read preserves storage and caller-request refusals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalTransactionReadDenial {
    Conflict(crate::transactions::data::CommitConflict),
    StorageUnavailable(crate::storage::data::RelationalStorageUnavailable),
    Request(crate::execution::RelationalExecutionDenialCause),
}

impl From<crate::transactions::data::CommitConflict> for RelationalTransactionReadDenial {
    fn from(conflict: crate::transactions::data::CommitConflict) -> Self {
        Self::Conflict(conflict)
    }
}

impl From<crate::storage::residency::RelationalResidencyDemandFailure>
    for RelationalTransactionReadDenial
{
    fn from(failure: crate::storage::residency::RelationalResidencyDemandFailure) -> Self {
        match failure {
            crate::storage::residency::RelationalResidencyDemandFailure::StorageUnavailable(
                failure,
            ) => Self::StorageUnavailable(failure),
            crate::storage::residency::RelationalResidencyDemandFailure::Request(cause) => {
                Self::Request(cause)
            }
        }
    }
}
