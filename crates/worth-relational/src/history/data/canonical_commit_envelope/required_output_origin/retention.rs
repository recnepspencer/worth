use super::*;
pub(super) fn origin_request(
    cause: impl Into<crate::execution::denial_cause::RelationalExecutionDenialCause>,
) -> TransactionCommitError {
    TransactionCommitError::execution(crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
        kind: crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenialKind::Cause(cause.into()),
        partition_identity: None,
    })
}
