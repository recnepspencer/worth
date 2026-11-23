/// Replay retention refuses before installing a holder or changing its pins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalReplayRetentionDenial {
    Request(crate::execution::denial_cause::RelationalExecutionDenialCause),
    StorageUnavailable(crate::storage::data::RelationalStorageUnavailable),
}

impl From<crate::execution::denial_cause::RelationalExecutionDenialCause>
    for RelationalReplayRetentionDenial
{
    fn from(cause: crate::execution::denial_cause::RelationalExecutionDenialCause) -> Self {
        match cause {
            crate::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable(
                failure,
            ) => Self::StorageUnavailable(failure),
            cause => Self::Request(cause),
        }
    }
}

impl From<crate::storage::data::RelationalStorageUnavailable> for RelationalReplayRetentionDenial {
    fn from(cause: crate::storage::data::RelationalStorageUnavailable) -> Self {
        Self::StorageUnavailable(cause)
    }
}
