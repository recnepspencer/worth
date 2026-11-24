use super::outcomes::transaction_commit_error::TransactionCommitError;
use crate::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;

pub fn copy_overflow() -> TransactionCommitError {
    TransactionCommitError::execution(
        crate::execution::denial_cause::RelationalExecutionDenialCause::ChargedBytesOverflow.into(),
    )
}
