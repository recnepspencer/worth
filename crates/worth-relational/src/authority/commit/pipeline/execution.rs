use super::artifact_execution::assemble_commit_artifacts;
use super::authority_context::AuthoritativeCommitContext;
use super::boundary_validation::validate_commit_boundary;
use super::draft_execution::prepare_commit_execution;
use super::execution_admission::admit_commit_execution;
use super::history_binding::bind_commit_history;
use super::mutation_execution::mutate_commit_execution;
use super::publication_execution::prepare_commit_publication_execution;
use super::snapshot_validation::validate_snapshot_publication;
use crate::transactions::data::{CommitResult, TransactionCommitError};

pub(crate) fn prepare_consulted_authoritative_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
    let allocation_policy = execution.allocation_policy();
    let scope_control = crate::validation::engine::InvariantPreparationControl::for_transaction(
        &context.validation_input,
        allocation_policy,
        crate::mvcc::RelationalInterruptionBoundary::CandidatePreparation,
    );
    scope_control.check()?;
    let runtime_instance_id = runtime.runtime_instance_id();
    let publication_binding = runtime.publication_binding();
    let transaction_id = context.transaction_id;
    let branch_id = context.validation_input.target_branch().clone();
    let control = context.validation_input.control().clone();
    let retention_binding = context
        .validation_input
        .basis()
        .inner
        .retention_binding
        .clone();
    let expected_basis = context.validation_input.basis().descriptor().clone();
    let expected_root = std::sync::Arc::clone(&context.validation_input.basis().inner.root);
    if let Some(interruption) =
        control.observe(crate::mvcc::RelationalInterruptionBoundary::CandidatePreparation)
    {
        retention_binding.record_interruption(interruption);
        return Err(TransactionCommitError::interrupted(interruption));
    }
    require_parent_publication_settled(runtime, &expected_root)?;
    let publication_cell = runtime
        .history
        .branch_cell(&branch_id)
        .expect("admitted commit branch remains registered")
        .publication_cell();
    let diagnostic_capture = runtime.diagnostics.begin_operation_capture();
    let admitted = admit_commit_execution(runtime, context)?;
    let prepared = prepare_commit_execution(execution, runtime, admitted)?;
    let boundary_validated =
        validate_commit_boundary(execution, runtime, prepared, &scope_control)?;
    let mutated = mutate_commit_execution(execution, runtime, boundary_validated, &scope_control)?;
    let history_bound = bind_commit_history(execution, runtime, mutated)?;
    let snapshot_validated =
        validate_snapshot_publication(execution, runtime, history_bound, &scope_control)?;
    let assembled = assemble_commit_artifacts(execution, runtime, snapshot_validated)?;
    let mut prepared = prepare_commit_publication_execution(execution, runtime, assembled)?;
    prepared.append_diagnostics(diagnostic_capture.finish());
    if let Some(interruption) =
        control.observe(crate::mvcc::RelationalInterruptionBoundary::CandidatePreparation)
    {
        retention_binding.record_interruption(interruption);
        return Err(TransactionCommitError::interrupted(interruption));
    }
    super::consult_stage(execution, super::CommitStage::CandidateRegistration)?;
    let published_snapshot_slot =
        runtime
            .reserve_published_snapshot_slot()
            .map_err(|maximum_handles| {
                TransactionCommitError::publication_deferred(
                crate::mvcc::RelationalPublicationDeferred::PublishedSnapshotCapacityExhausted {
                    maximum_handles,
                },
            )
            })?;
    let candidate_retention =
        crate::history::retention::RelationalCandidateRetentionObligation::acquire(
            &retention_binding,
            publication_cell.identity().clone(),
            std::sync::Arc::clone(&expected_root),
            std::sync::Arc::clone(prepared.prepared_root()),
        )
        .map_err(|denial| match denial {
            crate::history::retention::RelationalRetentionAcquisitionDenial::CapacityExhausted => {
                TransactionCommitError::publication_deferred(
                    crate::mvcc::RelationalPublicationDeferred::RetentionBackpressure,
                )
            }
            crate::history::retention::RelationalRetentionAcquisitionDenial::OwnerUnavailable => {
                TransactionCommitError::publication_denied(
                    crate::mvcc::RelationalPublicationDenial::OwnerUnavailable {
                        runtime_instance_id,
                    },
                )
            }
            crate::history::retention::RelationalRetentionAcquisitionDenial::IdentityExhausted => {
                TransactionCommitError::publication_failed(
                    crate::mvcc::RelationalPublicationFailure::new(
                        crate::mvcc::RelationalPublicationFailureKind::RetentionIdentityExhausted,
                        "prepared candidate retention identity exhausted before effects",
                    ),
                )
            }
            other => TransactionCommitError::publication_failed(
                crate::mvcc::RelationalPublicationFailure::new(
                    crate::mvcc::RelationalPublicationFailureKind::RetentionOwner,
                    format!("prepared candidate retention admission failed: {other:?}"),
                ),
            ),
        })?;
    if let Some(interruption) =
        control.observe(crate::mvcc::RelationalInterruptionBoundary::CandidatePreparation)
    {
        retention_binding.record_interruption(interruption);
        return Err(TransactionCommitError::interrupted(interruption));
    }
    prepared.release_transaction_retention();
    let candidate = crate::mvcc::PreparedRelationalCommitCandidate::new(
        runtime_instance_id,
        runtime.schema_contract_runtime.custom_invariant_generation,
        publication_binding,
        transaction_id,
        branch_id,
        expected_basis,
        expected_root,
        publication_cell,
        prepared,
        candidate_retention,
        published_snapshot_slot,
        control,
        runtime
            .config
            .publication
            .policy
            .candidate_max_lifetime_millis,
        runtime.config.publication.policy.max_prepared_candidates,
    )
    .map_err(|stop| match stop {
        crate::mvcc::PreparedRelationalCandidateAdmissionStop::Deferred(deferred) => {
            TransactionCommitError::publication_deferred(deferred)
        }
        crate::mvcc::PreparedRelationalCandidateAdmissionStop::Failed(failure) => {
            TransactionCommitError::publication_failed(failure)
        }
    })?;
    runtime
        .history
        .record_candidate_preparation(candidate.branch());
    Ok(candidate)
}

/// The borrowed request proves custody through publication; it is never retained.
pub(crate) fn publish_prepared_authoritative_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalRuntime,
    candidate: crate::mvcc::PreparedRelationalCommitCandidate,
) -> Result<CommitResult, TransactionCommitError> {
    let outcome = runtime
        .publication_port()
        .compare_and_publish(execution, candidate);
    let performed = match outcome {
        crate::mvcc::RelationalPublicationOutcome::RequestStopped(stop) => {
            return Err(TransactionCommitError::execution(stop.into()));
        }
        crate::mvcc::RelationalPublicationOutcome::Performed(performed) => performed,
        crate::mvcc::RelationalPublicationOutcome::Stale(stale) => {
            return Err(TransactionCommitError::conflict(
                crate::transactions::data::CommitConflict::new(
                    crate::transactions::data::ConflictClass::StaleValidationBasis {
                        detail: format!("branch moved before publication: {stale:?}"),
                    },
                ),
            ));
        }
        crate::mvcc::RelationalPublicationOutcome::Denied(
            crate::mvcc::RelationalPublicationDenial::ForeignRuntime {
                expected_runtime_instance_id,
                actual_runtime_instance_id,
            },
        ) => {
            return Err(TransactionCommitError::conflict(
                crate::transactions::data::CommitConflict::new(
                    crate::transactions::data::ConflictClass::ForeignRuntime {
                        expected_runtime_instance_id,
                        actual_runtime_instance_id,
                    },
                ),
            ));
        }
        crate::mvcc::RelationalPublicationOutcome::Denied(denial) => {
            return Err(TransactionCommitError::publication_denied(denial));
        }
        crate::mvcc::RelationalPublicationOutcome::Interrupted(interruption) => {
            return Err(TransactionCommitError::interrupted(interruption));
        }
        crate::mvcc::RelationalPublicationOutcome::Deferred(deferred) => {
            return Err(TransactionCommitError::publication_deferred(deferred));
        }
        crate::mvcc::RelationalPublicationOutcome::Failed(failure) => {
            return Err(TransactionCommitError::publication_failed(failure));
        }
    };
    runtime.settle_performed_publication(performed)
}

fn require_parent_publication_settled(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    expected_root: &std::sync::Arc<crate::branch::RelationalBranchRoot>,
) -> Result<(), TransactionCommitError> {
    let Some(parent) = expected_root.canonical_envelope() else {
        return Ok(());
    };
    let commit_id = parent.commit.commit_id;
    if !runtime.history.publication_requires_settlement(commit_id) {
        return Ok(());
    }
    Err(publication_outcome_error(format!(
        "parent commit {} requires explicit owner settlement",
        commit_id.0
    )))
}

fn publication_outcome_error(detail: String) -> TransactionCommitError {
    TransactionCommitError::publication(crate::publication::data::PublicationError::new(
        crate::publication::bundle::PublicationStage::Visibility,
        detail,
    ))
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
    let candidate = prepare_consulted_authoritative_commit(execution, &preparation, context)?;
    publish_prepared_authoritative_commit(execution, runtime, candidate)
}
