use crate::transactions::data::{
    CommitConflict, CommitLog, CommitPhase, ConflictClass, TransactionCommitError,
};
use std::time::Instant;

pub(super) fn invalid_merge_context(detail: &str) -> TransactionCommitError {
    TransactionCommitError::conflict(CommitConflict::new(ConflictClass::InvalidMergeParent {
        detail: detail.to_string(),
    }))
}

pub(super) fn stale_strategy_validation_basis(detail: &str) -> TransactionCommitError {
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
        TransactionCommitError::PerformedButDurabilityDeferred { .. } => {}
    }
    error.with_commit_log(commit_log.clone())
}

pub(super) fn elapsed_micros(started: Instant) -> u64 {
    started.elapsed().as_micros() as u64
}
