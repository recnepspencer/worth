//! Preserve request control while retaining the replay failure for bad evidence.
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_preparation::CommitPreparationErrorOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::transactions::data::outcomes::transaction_commit_error::{
    CommitExecutionDenialKind, TransactionCommitError,
};

pub(crate) fn replay_commit_stop(
    error: TransactionCommitError,
    detail: String,
) -> RelationalRecoveryStop {
    match error {
        TransactionCommitError::Execution { denial, .. } => match denial.kind {
            CommitExecutionDenialKind::Cause(Cause::StorageUnavailable(failure)) => failure.into(),
            CommitExecutionDenialKind::Cause(cause) => RelationalRecoveryStop::Request(cause),
        },
        TransactionCommitError::Interrupted { interruption, .. } => {
            RelationalRecoveryStop::Request(match interruption.interruption() {
                crate::runtime::operation_control::interruption_event::RelationalOperationInterruption::Cancelled => Cause::Cancelled,
                crate::runtime::operation_control::interruption_event::RelationalOperationInterruption::TimedOut => Cause::DeadlineElapsed,
            })
        }
        TransactionCommitError::Conflict { error, .. } => match &error.class {
            crate::transactions::data::outcomes::conflict_class::ConflictClass::ContentAdmission { denial } => (*denial).into(),
            _ => match error.allocation_denial() {
            Some(denial) => crate::symbols::data::field_text_recovery_stop(
                crate::symbols::data::RelationalFieldTextAdmissionDenial::Allocation(
                    denial.clone(),
                ),
            ),
            None => DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail).into(),
            },
        },
        TransactionCommitError::Preparation { error, .. } => match error.reason() {
            crate::transactions::data::outcomes::commit_preparation::CommitPreparationReason::StorageUnavailable(failure) => failure.into(),
            _ => DurabilityError::new(RecoveryFailureClass::ReplayFailure, detail).into(),
        },
        TransactionCommitError::Publication { .. }
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
    fn execution_storage_refusal_is_cold_storage_failure_in_tail_replay() {
        let failure = crate::storage::data::RelationalStorageUnavailable {
            subject: crate::storage::data::RelationalStorageSubject::Region(
                crate::identity::data::PartitionId::main(),
            ),
            cause: crate::storage::data::RelationalStorageUnavailableCause::Damaged,
        };
        let error = TransactionCommitError::execution(
            crate::transactions::data::outcomes::transaction_commit_error::CommitExecutionDenial {
                kind: CommitExecutionDenialKind::Cause(Cause::StorageUnavailable(failure)),
                partition_identity: None,
            },
        );
        let RelationalRecoveryStop::Durability(error) =
            replay_commit_stop(error, "storage read refused".into())
        else {
            panic!("storage refusal is a durable-source failure");
        };
        assert_eq!(error.class, RecoveryFailureClass::ColdStorageUnavailable);
        assert_eq!(error.storage_unavailable, Some(failure));
    }
    #[test]
    fn replay_interrupted_commit_preserves_each_control_cause_by_value() {
        use crate::runtime::operation_control::interruption_event::{
            RelationalInterruptionBoundary, RelationalInterruptionEvent,
        };
        for (stop, cause) in [
            (worth_execution::MapKernelStop::Cancelled, Cause::Cancelled),
            (
                worth_execution::MapKernelStop::DeadlineElapsed,
                Cause::DeadlineElapsed,
            ),
        ] {
            let boundary = RelationalInterruptionBoundary::ProposalValidation;
            let event = RelationalInterruptionEvent::from_request_stop(stop, boundary).unwrap();
            let stop = replay_commit_stop(
                TransactionCommitError::interrupted(event),
                "not corrupt evidence".into(),
            );
            assert_eq!(stop, RelationalRecoveryStop::Request(cause));
        }
    }
    #[test]
    fn replay_conflict_keeps_its_durability_failure_by_value() {
        let conflict = crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
            crate::transactions::data::outcomes::conflict_class::ConflictClass::InvalidMergeParent {
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
