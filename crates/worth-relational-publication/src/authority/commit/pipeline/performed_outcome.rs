//! Translate the exact native publication outcome before owner settlement.
use crate::mvcc::publication::outcome::{PerformedRelationalCommit, RelationalPublicationOutcome};
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_contracts::internal_port as contracts;

pub fn require_performed(
    outcome: RelationalPublicationOutcome,
) -> Result<PerformedRelationalCommit, TransactionCommitError> {
    let performed = match outcome {
        RelationalPublicationOutcome::RequestStopped(stop) => {
            return Err(TransactionCommitError::execution(stop.into()));
        }
        RelationalPublicationOutcome::Performed(performed) => performed,
        RelationalPublicationOutcome::Stale(stale) => {
            return Err(TransactionCommitError::conflict(
                contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                    contracts::transactions::data::outcomes::conflict_class::ConflictClass::StaleValidationBasis {
                        detail: format!("branch moved before publication: {stale:?}"),
                    },
                ),
            ));
        }
        RelationalPublicationOutcome::Denied(
            contracts::mvcc::publication::outcome::refusals::RelationalPublicationDenial::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            },
        ) => {
            return Err(TransactionCommitError::conflict(
                contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                    contracts::transactions::data::outcomes::conflict_class::ConflictClass::ForeignRuntime {
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
