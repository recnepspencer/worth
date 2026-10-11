//! Preserve request control while retaining the replay failure for bad evidence.
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::{CommitExecutionDenialKind, TransactionCommitError};

pub(crate) fn replay_commit_stop(
    error: TransactionCommitError,
    detail: String,
) -> RelationalRecoveryStop {
    match error {
        TransactionCommitError::Execution { denial, .. } => match denial.kind {
            CommitExecutionDenialKind::Cause(cause) => RelationalRecoveryStop::Request(cause),
        },
        TransactionCommitError::Interrupted { interruption, .. } => {
            RelationalRecoveryStop::Request(match interruption.interruption() {
                crate::mvcc::RelationalOperationInterruption::Cancelled => Cause::Cancelled,
                crate::mvcc::RelationalOperationInterruption::TimedOut => Cause::DeadlineElapsed,
            })
        }
        TransactionCommitError::Conflict { .. }
        | TransactionCommitError::Publication { .. }
        | TransactionCommitError::Preparation { .. }
        | TransactionCommitError::PublicationDenied { .. }
        | TransactionCommitError::PublicationDeferred { .. }
        | TransactionCommitError::PublicationFailed { .. }
        | TransactionCommitError::PerformedButDurabilityDeferred { .. } => {
            DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail).into()
        }
    }
}
