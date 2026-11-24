//! Translate the exact native publication outcome before owner settlement.
use worth_relational_contracts::internal_port::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use crate::mvcc::publication::outcome::{PerformedRelationalCommit, RelationalPublicationOutcome};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

pub(crate) fn require_performed(
    outcome: RelationalPublicationOutcome,
) -> Result<PerformedRelationalCommit, TransactionCommitError> {
    let performed = match outcome {
        RelationalPublicationOutcome::RequestStopped(stop) => {
            return Err(TransactionCommitError::execution(stop.into()));
        }
        RelationalPublicationOutcome::Performed(performed) => performed,
        RelationalPublicationOutcome::Stale(stale) => {
            return Err(TransactionCommitError::conflict(
                crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                    crate::transactions::data::outcomes::conflict_class::ConflictClass::StaleValidationBasis {
                        detail: format!("branch moved before publication: {stale:?}"),
                    },
                ),
            ));
        }
        RelationalPublicationOutcome::Denied(
            crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            },
        ) => {
            return Err(TransactionCommitError::conflict(
                crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                    crate::transactions::data::outcomes::conflict_class::ConflictClass::ForeignRuntime {
                        expected_runtime_instance_id,
                        actual_runtime_instance_id,
                    },
                ),
            ));
        }
        RelationalPublicationOutcome::Denied(denial) => {
            return Err(TransactionCommitError::publication_denied(denial));
        }
        RelationalPublicationOutcome::Interrupted(interruption) => {
            return Err(TransactionCommitError::interrupted(interruption));
        }
        RelationalPublicationOutcome::Deferred(deferred) => {
            return Err(TransactionCommitError::publication_deferred(deferred));
        }
        RelationalPublicationOutcome::Failed(failure) => {
            return Err(TransactionCommitError::publication_failed(failure));
        }
        RelationalPublicationOutcome::DurabilityUnresolved(unresolved) => {
            return Err(TransactionCommitError::durability_unresolved(unresolved));
        }
    };
    Ok(performed)
}
