use super::outcomes::transaction_commit_error::TransactionCommitError;

pub(crate) fn copy_overflow() -> TransactionCommitError {
    TransactionCommitError::execution(
        crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow.into(),
    )
}
