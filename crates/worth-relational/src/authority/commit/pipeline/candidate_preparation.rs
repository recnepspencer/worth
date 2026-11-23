//! Move the original history proof into candidate registration after early frames expire.
use super::authority_context::AuthoritativeCommitContext;
use super::history_preparation::prepare_bound_history;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
pub(crate) mod registration;
mod stages;
use super::phase_backing::CommitPhaseBacking;

pub(crate) struct SelectedCandidateSymbols {
    pub(crate) dictionary:
        crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    pub(crate) references: std::collections::BTreeSet<crate::symbols::data::Symbol>,
}

pub(crate) struct CandidateArtifactSelection<'a> {
    pub(crate) owners: super::artifact_execution::preparation::PublicationArtifactOwners<'a>,
    pub(crate) max_patch_records: Option<usize>,
}

struct HistoryPreparedCandidate<'scope, 'authority> {
    scope_control: crate::validation::engine::InvariantPreparationControl<'scope, 'authority>,
    history_bound: CommitPhaseBacking<super::history_binding::HistoryBoundCommitExecution>,
    runtime_instance_id: u64,
    publication_binding: crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding,
    transaction_id: crate::transactions::data::primitives::TransactionId,
    branch_id: crate::history::data::BranchId,
    control: crate::runtime::operation_control::RelationalOperationControl,
    retention_binding: crate::history::retention::RelationalBranchRetentionBinding,
    decision_basis: crate::branch::basis::AdmittedRelationalBranchBasis,
    coupled_proposal:
        Option<crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>,
    expected_basis: crate::branch::basis::RelationalBranchBasisDescriptor,
    expected_root: std::sync::Arc<crate::branch::root::RelationalBranchRoot>,
    publication_cell: crate::branch::reference_publication_cell::RelationalBranchPublicationCell,
    required_output_publication: Option<
        crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission,
    >,
    required_output_materialization:
        Option<std::sync::Arc<crate::history::data::CanonicalRequiredOutputMaterialization>>,
    caller_association: Option<crate::history::data::RelationalCallerAssociation>,
    diagnostic_capture: crate::publication::diagnostic_store::RelationalDiagnosticOperationCapture,
}

pub(crate) fn prepare_consulted_authoritative_commit(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut context: AuthoritativeCommitContext,
) -> Result<crate::mvcc::publication::PreparedRelationalCommitCandidate, TransactionCommitError> {
    if matches!(&context.authority_input,
        super::authority_context::CommitAuthorityInput::Mutation(plan) if plan.retained().is_some())
        && runtime.durability_admission.has_attached_medium()
    {
        return Err(TransactionCommitError::publication_denied(
            crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial::RetainedEntitySelectionDurabilityUnsupported,
        ));
    }
    runtime
        .durability_admission
        .admit_home_program_write(
            context
                .validation_input
                .home_step()
                .map(std::sync::Arc::as_ref),
            context
                .validation_input
                .recovered_codec_publication_custody()
                .map(std::sync::Arc::as_ref),
        )
        .map_err(|cause| {
            TransactionCommitError::publication_denied(
                crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial::HomeProgramTransition(cause),
            )
        })?;
    if let Some(step) = context.validation_input.home_step()
        && !context
            .caller_association
            .as_ref()
            .is_some_and(|association| step.admits_association(association.as_bytes()))
    {
        return Err(TransactionCommitError::publication_denied(
            crate::mvcc::publication::outcome::refusals::RelationalPublicationDenial::HomeProgramTransition(
                crate::durability::medium::home_write_denial::RelationalHomeProgramWriteDenial::TransitionRequired,
            ),
        ));
    }
    let original = context
        .recovered_index_catalog
        .take()
        .map(|catalog| catalog.original_for(runtime.runtime_instance_id()))
        .transpose()?;
    let history = prepare_history(execution, runtime, context)?;
    let (symbols, validated) = stages::validate(execution, history)?;
    let artifact_owners = runtime.bind_candidate_artifacts(&symbols, original.as_ref());
    let assembled = stages::assemble(execution, artifact_owners, validated)?;
    let publication_owners = runtime.publication_phase_owners(&symbols.dictionary);
    let published = stages::publish(execution, publication_owners, assembled)?;
    stages::register(
        execution,
        runtime.candidate_registration_inputs(&symbols.dictionary),
        published,
    )
}

// Later publication locals must not occupy the deep admission/draft call's frame.
#[inline(never)]
fn prepare_history<'scope, 'authority>(
    execution: &worth_execution::ExecutionRequest<'scope, 'authority>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut context: AuthoritativeCommitContext,
) -> Result<CommitPhaseBacking<HistoryPreparedCandidate<'scope, 'authority>>, TransactionCommitError>
{
    let required_output_publication = context.required_output_publication.take();
    let caller_association = context.caller_association.take();
    let required_output_materialization = context.required_output_materialization.take();
    let allocation_policy = execution.allocation_policy();
    let scope_control = crate::validation::engine::InvariantPreparationControl::for_transaction(
        &context.validation_input,
        allocation_policy,
        crate::runtime::operation_control::interruption_event::RelationalInterruptionBoundary::CandidatePreparation,
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
    let decision_basis = context.validation_input.basis().clone();
    let coupled_proposal = context.validation_input.coupled_proposal().cloned();
    let expected_basis = decision_basis.descriptor().clone();
    let expected_root = std::sync::Arc::clone(&context.validation_input.basis().inner.root);
    if let Some(interruption) =
        control.observe(crate::runtime::operation_control::interruption_event::RelationalInterruptionBoundary::CandidatePreparation)
    {
        retention_binding.record_interruption(interruption);
        return Err(TransactionCommitError::interrupted(interruption));
    }
    require_parent_publication_settled(&runtime.history, &expected_root)?;
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
    let history_bound = prepare_bound_history(execution, runtime, context)?;
    CommitPhaseBacking::new(
        execution,
        HistoryPreparedCandidate {
            scope_control,
            history_bound,
            runtime_instance_id,
            publication_binding,
            transaction_id,
            branch_id,
            control,
            retention_binding,
            decision_basis,
            coupled_proposal,
            expected_basis,
            expected_root,
            publication_cell,
            required_output_publication,
            required_output_materialization,
            caller_association,
            diagnostic_capture,
        },
    )
}

fn require_parent_publication_settled(
    history: &crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory,
    expected_root: &std::sync::Arc<crate::branch::root::RelationalBranchRoot>,
) -> Result<(), TransactionCommitError> {
    let Some(parent) = expected_root.commit_receipt() else {
        return Ok(());
    };
    let commit_id = parent.commit_id;
    if !history.publication_requires_settlement(commit_id) {
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
