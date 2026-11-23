use super::authority_context::AuthoritativeCommitContext;
use crate::transactions::data::outcomes::commit_result::CommitResult;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

/// The borrowed request proves custody through publication; it is never retained.
pub(crate) fn publish_prepared_authoritative_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    candidate: crate::mvcc::publication::PreparedRelationalCommitCandidate,
) -> Result<CommitResult, TransactionCommitError> {
    let outcome = runtime
        .publication_port()
        .compare_and_publish(execution, candidate);
    let performed = match outcome {
        crate::mvcc::publication::outcome::RelationalPublicationOutcome::RequestStopped(stop) => {
            return Err(TransactionCommitError::execution(stop.into()));
        }
        crate::mvcc::publication::outcome::RelationalPublicationOutcome::Performed(performed) => performed,
        crate::mvcc::publication::outcome::RelationalPublicationOutcome::Stale(stale) => {
            return Err(TransactionCommitError::conflict(
                crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
                    crate::transactions::data::outcomes::conflict_class::ConflictClass::StaleValidationBasis {
                        detail: format!("branch moved before publication: {stale:?}"),
                    },
                ),
            ));
        }
        crate::mvcc::publication::outcome::RelationalPublicationOutcome::Denied(
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
        crate::mvcc::publication::outcome::RelationalPublicationOutcome::Denied(denial) => {
            return Err(TransactionCommitError::publication_denied(denial));
        }
        crate::mvcc::publication::outcome::RelationalPublicationOutcome::Interrupted(interruption) => {
            return Err(TransactionCommitError::interrupted(interruption));
        }
        crate::mvcc::publication::outcome::RelationalPublicationOutcome::Deferred(deferred) => {
            return Err(TransactionCommitError::publication_deferred(deferred));
        }
        crate::mvcc::publication::outcome::RelationalPublicationOutcome::Failed(failure) => {
            return Err(TransactionCommitError::publication_failed(failure));
        }
        crate::mvcc::publication::outcome::RelationalPublicationOutcome::DurabilityUnresolved(unresolved) => {
            return Err(TransactionCommitError::durability_unresolved(unresolved));
        }
    };
    runtime.settle_performed_publication(performed)
}

pub(crate) fn execute_authoritative_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    context: AuthoritativeCommitContext,
) -> Result<CommitResult, TransactionCommitError> {
    super::consult_stage(execution, super::CommitStage::Admission)?;
    let preparation = runtime.preparation_runtime_snapshot();
    // A detached validated proposal retains its account; merge and replay
    // start theirs here. Every charged dispatch then spends the same ceiling.
    let preparation = if execution.is_leased() {
        preparation.with_commit_work_budget(context.commit_work_budget.clone())
    } else {
        preparation
    };
    let candidate = super::candidate_preparation::prepare_consulted_authoritative_commit(
        execution,
        &preparation,
        context,
    )?;
    publish_prepared_authoritative_commit(execution, runtime, candidate)
}
