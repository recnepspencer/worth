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
        TransactionCommitError::Conflict { error, .. } => match error.allocation_denial() {
            Some(denial) => crate::symbols::data::field_text_recovery_stop(
                crate::symbols::data::RelationalFieldTextAdmissionDenial::Allocation(
                    denial.clone(),
                ),
            ),
            None => DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail).into(),
        },
        TransactionCommitError::Publication { .. }
        | TransactionCommitError::Preparation { .. }
        | TransactionCommitError::PublicationDenied { .. }
        | TransactionCommitError::PublicationDeferred { .. }
        | TransactionCommitError::PublicationFailed { .. }
        // Replay runs before a medium is attached, so this cannot occur; it
        // would still be replay failure rather than a durable fact.
        | TransactionCommitError::DurabilityUnresolved { .. } => {
            DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail).into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_interrupted_commit_preserves_each_control_cause_by_value() {
        use crate::mvcc::{
            RelationalInterruptionBoundary, RelationalOperationControl,
            RelationalOperationInterruption as Interrupt,
        };
        for (interruption, cause) in [
            (Interrupt::Cancelled, Cause::Cancelled),
            (Interrupt::TimedOut, Cause::DeadlineElapsed),
        ] {
            let boundary = RelationalInterruptionBoundary::ProposalValidation;
            let event = RelationalOperationControl::uninterrupted()
                .with_injected_interruption_for_test(boundary, interruption, 1)
                .observe(boundary)
                .unwrap();
            let stop = replay_commit_stop(
                TransactionCommitError::interrupted(event),
                "not corrupt evidence".into(),
            );
            assert_eq!(stop, RelationalRecoveryStop::Request(cause));
        }
    }
    #[test]
    fn replay_conflict_keeps_its_durability_failure_by_value() {
        let conflict = crate::transactions::data::CommitConflict::new(
            crate::transactions::data::ConflictClass::InvalidMergeParent {
                detail: "corrupt merge".into(),
            },
        );
        assert_eq!(
            replay_commit_stop(
                TransactionCommitError::conflict(conflict),
                "corrupt envelope".into()
            ),
            RelationalRecoveryStop::Durability(DurabilityError::new(
                RecoveryFailureClass::ReplayFailure,
                "corrupt envelope"
            ))
        );
    }
}
