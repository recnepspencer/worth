use worth_relational_contracts::internal_port::execution::monotonic_time::elapsed_since;
use contracts::transactions::data::commit_log::CommitLog;
use contracts::transactions::data::commit_log::CommitPhase;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflict;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::conflict_class::ConflictClass;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_foundational::time::MonotonicInstant;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn stale_proposal_validation_basis(detail: &str) -> TransactionCommitError {
    TransactionCommitError::conflict(CommitConflict::new(ConflictClass::StaleValidationBasis {
        detail: detail.to_string(),
    }))
}

pub(super) fn attach_rejection(
    commit_log: &mut CommitLog,
    phase: CommitPhase,
    error: TransactionCommitError,
) -> TransactionCommitError {
    match &error {
        TransactionCommitError::Conflict { error, .. } => {
            commit_log.record_rejection(phase, Some(error.code()), None, error.detail());
        }
        TransactionCommitError::Publication { error, .. } => {
            commit_log.record_rejection(phase, None, Some(error.stage), error.detail.clone());
        }
        TransactionCommitError::Preparation { error, .. } => {
            commit_log.record_rejection(phase, Some(error.code()), None, error.detail());
        }
        TransactionCommitError::Interrupted { .. } | TransactionCommitError::Execution { .. } => {
            commit_log.record_rejection(phase, None, None, error.detail());
        }
        TransactionCommitError::PublicationDenied { .. }
        | TransactionCommitError::PublicationDeferred { .. }
        | TransactionCommitError::PublicationFailed { .. } => {
            commit_log.record_rejection(phase, None, None, error.detail());
        }
        // Not a rejection: the commit's durable fate is not yet known.
        TransactionCommitError::DurabilityUnresolved { .. } => {}
    }
    error.with_commit_log(commit_log.clone())
}

pub(super) fn elapsed_micros(started: MonotonicInstant) -> u64 {
    elapsed_since(started).as_micros() as u64
}
