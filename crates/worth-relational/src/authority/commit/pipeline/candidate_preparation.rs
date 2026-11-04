//! Move the original history proof into candidate registration after early frames expire.
use super::artifact_execution::assemble_commit_artifacts;
use super::authority_context::AuthoritativeCommitContext;
use super::history_preparation::prepare_bound_history;
use super::publication_execution::prepare_commit_publication_execution;
use super::snapshot_validation::validate_snapshot_publication;
use crate::transactions::data::TransactionCommitError;
mod registration;

struct HistoryPreparedCandidate<'scope, 'authority> {
    recovered_runtime: crate::runtime::RelationalPreparationRuntime,
    scope_control: crate::validation::engine::InvariantPreparationControl<'scope, 'authority>,
    history_bound: super::history_binding::HistoryBoundCommitExecution,
    runtime_instance_id: u64,
    publication_binding: crate::runtime::RelationalRuntimePublicationBinding,
    transaction_id: crate::transactions::data::TransactionId,
    branch_id: crate::history::data::BranchId,
    control: crate::mvcc::RelationalOperationControl,
    retention_binding: crate::history::retention::RelationalBranchRetentionBinding,
    expected_basis: crate::branch::RelationalBranchBasisDescriptor,
    expected_root: std::sync::Arc<crate::branch::RelationalBranchRoot>,
    publication_cell: crate::branch::RelationalBranchPublicationCell,
    required_output_publication: Option<crate::mvcc::RequiredOutputPublicationAdmission>,
    required_output_materialization:
        Option<std::sync::Arc<crate::history::data::CanonicalRequiredOutputMaterialization>>,
    caller_association: Option<crate::mvcc::RelationalCallerAssociation>,
    diagnostic_capture: crate::runtime::RelationalDiagnosticOperationCapture,
}

pub(crate) fn prepare_consulted_authoritative_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    context: AuthoritativeCommitContext,
) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
    let history = prepare_history(execution, runtime, context)?;
    register_candidate(execution, history)
}

// Later publication locals must not occupy the deep admission/draft call's frame.
#[inline(never)]
fn prepare_history<'scope, 'authority>(
    execution: &worth_execution::ExecutionRequest<'scope, 'authority>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut context: AuthoritativeCommitContext,
) -> Result<HistoryPreparedCandidate<'scope, 'authority>, TransactionCommitError> {
    let mut recovered_runtime = runtime.clone();
    if let Some(catalog) = context.recovered_index_catalog.take() {
        let (delta, authored) = catalog.original_for(runtime.runtime_instance_id())?;
        recovered_runtime.recovered_index_catalog = Some(delta);
        recovered_runtime.recovered_authored_index_writes = Some(authored);
    }
    let runtime = &recovered_runtime;
    let required_output_publication = context.required_output_publication.take();
    let caller_association = context.caller_association.take();
    let required_output_materialization = context.required_output_materialization.take();
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
    if let Some(admission) = &required_output_publication {
        admission.validate_before_preparation(execution, context.validation_input.basis())?;
    }
    let publication_cell = runtime
        .history
        .branch_cell(&branch_id)
        .ok_or_else(|| {
            publication_outcome_error(format!(
                "admitted branch {} is no longer registered",
                branch_id.0
            ))
        })?
        .publication_cell();
    let diagnostic_capture = runtime.diagnostics.begin_operation_capture();
    let history_bound = prepare_bound_history(execution, runtime, context, &scope_control)?;
    Ok(HistoryPreparedCandidate {
        recovered_runtime,
        scope_control,
        history_bound,
        runtime_instance_id,
        publication_binding,
        transaction_id,
        branch_id,
        control,
        retention_binding,
        expected_basis,
        expected_root,
        publication_cell,
        required_output_publication,
        required_output_materialization,
        caller_association,
        diagnostic_capture,
    })
}

#[inline(never)]
fn register_candidate(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    history: HistoryPreparedCandidate<'_, '_>,
) -> Result<crate::mvcc::PreparedRelationalCommitCandidate, TransactionCommitError> {
    let HistoryPreparedCandidate {
        recovered_runtime,
        scope_control,
        history_bound,
        runtime_instance_id,
        publication_binding,
        transaction_id,
        branch_id,
        control,
        retention_binding,
        expected_basis,
        expected_root,
        publication_cell,
        required_output_publication,
        required_output_materialization,
        caller_association,
        diagnostic_capture,
    } = history;
    let runtime = &recovered_runtime;
    let final_runtime = runtime.with_candidate_symbols(history_bound.candidate_symbols());
    let runtime = &final_runtime;
    let snapshot_validated =
        validate_snapshot_publication(execution, runtime, history_bound, &scope_control)?;
    let assembled = assemble_commit_artifacts(execution, runtime, snapshot_validated)?;
    let mut prepared = prepare_commit_publication_execution(
        execution,
        runtime,
        assembled,
        required_output_publication,
        required_output_materialization,
        caller_association,
    )?;
    prepared.append_diagnostics(diagnostic_capture.finish());
    // Candidate registration owns counters, lifetime admission and final payload
    // placement only after Native artifact assembly has returned.
    registration::CandidateRegistration {
        runtime_instance_id,
        publication_binding,
        transaction_id,
        branch_id,
        control,
        retention_binding,
        expected_basis,
        expected_root,
        publication_cell,
    }
    .register(execution, runtime, prepared)
}

fn require_parent_publication_settled(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    expected_root: &std::sync::Arc<crate::branch::RelationalBranchRoot>,
) -> Result<(), TransactionCommitError> {
    let Some(parent) = expected_root.commit_receipt() else {
        return Ok(());
    };
    let commit_id = parent.commit_id;
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
