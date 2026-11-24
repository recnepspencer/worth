//! Move the original history proof into candidate registration after early frames expire.
use super::authority_context::AuthoritativeCommitContext;
use super::execution_admission::AdmittedCommitState;
use super::history_preparation::{prepare_bound_history, HistoryPreparationInputs};
use super::merge_draft::AcceptedMergeDraft;
use super::publication_validation::DraftValidationInput;
use crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
pub(crate) mod registration;
pub(super) mod stages;
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

/// The exact selected publication and history owners for early preparation.
pub(crate) struct CandidateHistoryInputs<'a, Draft> {
    pub(crate) runtime_identity: u64,
    pub(crate) publication: &'a RelationalRuntimePublicationBinding,
    pub(crate) history: &'a RelationalPreparationHistory,
    pub(crate) diagnostics:
        &'a crate::publication::diagnostic_store::RelationalDiagnosticArtifactStore,
    pub(crate) prepare: HistoryPreparationInputs<'a, Draft>,
}

pub(super) struct HistoryPreparedCandidate<'scope, 'authority> {
    scope_control: crate::validation::engine::InvariantPreparationControl<'scope, 'authority>,
    history_bound: CommitPhaseBacking<super::history_binding::HistoryBoundCommitExecution>,
    runtime_instance_id: u64,
    publication_binding: RelationalRuntimePublicationBinding,
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

// Later publication locals must not occupy the deep admission/draft call's frame.
#[inline(never)]
pub(super) fn prepare_history<'scope, 'authority, V: DraftValidationInput, Draft>(
    execution: &worth_execution::ExecutionRequest<'scope, 'authority>,
    inputs: CandidateHistoryInputs<'_, Draft>,
    mut context: AuthoritativeCommitContext<V>,
) -> Result<CommitPhaseBacking<HistoryPreparedCandidate<'scope, 'authority>>, TransactionCommitError>
where
    Draft: FnOnce(
        &ExecutionRequest<'_, '_>,
        &AdmittedCommitState<V>,
    ) -> Result<AcceptedMergeDraft, TransactionCommitError>,
{
    let required_output_publication = context.required_output_publication.take();
    let caller_association = context.caller_association.take();
    let required_output_materialization = context.required_output_materialization.take();
    let allocation_policy = execution.allocation_policy();
    let scope_control = crate::validation::engine::InvariantPreparationControl::for_transaction(
        crate::validation::engine::request::InvariantPreparationSource {
            operation: context.validation_input.control(),
            retention: &(*context.validation_input.basis().owner_inner()).retention_binding,
        },
        allocation_policy,
        crate::runtime::operation_control::interruption_event::RelationalInterruptionBoundary::CandidatePreparation,
    );
    scope_control.check()?;
    let runtime_instance_id = inputs.runtime_identity;
    let publication_binding = inputs.publication.clone();
    let transaction_id = context.transaction_id;
    let branch_id = context.validation_input.target_branch().clone();
    let control = context.validation_input.control().clone();
    let retention_binding = (*context.validation_input.basis().owner_inner())
        .retention_binding
        .clone();
    let decision_basis = context.validation_input.basis().clone();
    let coupled_proposal = context.validation_input.coupled_proposal().cloned();
    let expected_basis = decision_basis.descriptor().clone();
    let expected_root =
        std::sync::Arc::clone(&(*context.validation_input.basis().owner_inner()).root);
    if let Some(interruption) =
        control.observe(crate::runtime::operation_control::interruption_event::RelationalInterruptionBoundary::CandidatePreparation)
    {
        retention_binding.record_interruption(interruption);
        return Err(TransactionCommitError::interrupted(interruption));
    }
    require_parent_publication_settled(inputs.history, &expected_root)?;
    if let Some(admission) = &required_output_publication {
        admission.validate_before_preparation(execution, context.validation_input.basis())?;
    }
    let publication_cell = inputs
        .history
        .branch_cell(&branch_id)
        .ok_or_else(|| {
            publication_outcome_error(format!(
                "admitted branch {} is no longer registered",
                branch_id.0
            ))
        })?
        .publication_cell();
    let diagnostic_capture = inputs.diagnostics.begin_operation_capture();
    let history_bound = prepare_bound_history(execution, inputs.prepare, context)?;
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
    history: &RelationalPreparationHistory,
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
