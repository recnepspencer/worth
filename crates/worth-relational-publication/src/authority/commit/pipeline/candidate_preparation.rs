//! Move the original history proof into candidate registration after early frames expire.
use super::authority_context::AuthoritativeCommitContext;
use super::history_preparation::{prepare_bound_history, HistoryPreparationInputs};
use super::publication_validation::DraftValidationInput;
use crate::runtime::state::runtime_state::publication_lifecycle::RelationalRuntimePublicationBinding;
use crate::runtime::state::subsystems::history::history_preparation_binding::RelationalPreparationHistory;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;
pub mod registration;
pub mod stages;
use super::phase_backing::CommitPhaseBacking;

pub struct SelectedCandidateSymbols {
    pub dictionary:
        worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    pub references: std::collections::BTreeSet<contracts::symbols::data::symbol::Symbol>,
}

pub struct CandidateArtifactSelection<'a> {
    pub owners: super::artifact_execution::preparation::PublicationArtifactOwners<'a>,
    pub max_patch_records: Option<usize>,
}

/// The exact selected publication and history owners for early preparation.
pub struct CandidateHistoryInputs<'a> {
    pub runtime_identity: u64,
    pub publication: &'a RelationalRuntimePublicationBinding,
    pub history: &'a RelationalPreparationHistory,
    pub prepare: HistoryPreparationInputs<'a>,
}

pub struct HistoryPreparedCandidate<'scope, 'authority> {
    scope_control: mutation::validation::engine::request::preparation_control::InvariantPreparationControl<'scope, 'authority>,
    history_bound: CommitPhaseBacking<super::history_binding::HistoryBoundCommitExecution>,
    runtime_instance_id: u64,
    publication_binding: RelationalRuntimePublicationBinding,
    transaction_id: contracts::transactions::data::primitives::TransactionId,
    branch_id: contracts::history::data::coordinates::BranchId,
    control: contracts::runtime::operation_control::RelationalOperationControl,
    retention_binding: worth_relational_branch::internal_port::history::retention::lane::RelationalBranchRetentionBinding,
    decision_basis: worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis,
    coupled_proposal:
        Option<mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity>,
    expected_basis: worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor,
    expected_root: std::sync::Arc<worth_relational_branch::internal_port::branch::root::RelationalBranchRoot>,
    publication_cell: worth_relational_branch::internal_port::branch::reference_publication_cell::RelationalBranchPublicationCell,
    required_output_publication: Option<
        crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission,
    >,
    required_output_materialization:
        Option<std::sync::Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_materialization::CanonicalRequiredOutputMaterialization>>,
    caller_association: Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>,
}

// Later publication locals must not occupy the deep admission/draft call's frame.
#[inline(never)]
pub fn prepare_history<'scope, 'authority, V: DraftValidationInput>(
    execution: &worth_execution::ExecutionRequest<'scope, 'authority>,
    inputs: CandidateHistoryInputs<'_>,
    mut context: AuthoritativeCommitContext<V>,
) -> Result<CommitPhaseBacking<HistoryPreparedCandidate<'scope, 'authority>>, TransactionCommitError>
{
    let required_output_publication = context.required_output_publication.take();
    let caller_association = context.caller_association.take();
    let required_output_materialization = context.required_output_materialization.take();
    let allocation_policy = execution.allocation_policy();
    let scope_control = mutation::validation::engine::request::preparation_control::InvariantPreparationControl::for_transaction(
        mutation::validation::engine::request::preparation_control::InvariantPreparationSource {
            operation: context.validation_input.control(),
            retention: &context.validation_input.basis().owner_inner().retention_binding,
        },
        allocation_policy,
        contracts::runtime::operation_control::interruption_event::RelationalInterruptionBoundary::CandidatePreparation,
    );
    scope_control.check()?;
    let runtime_instance_id = inputs.runtime_identity;
    let publication_binding = inputs.publication.clone();
    let transaction_id = context.transaction_id;
    let branch_id = context.validation_input.target_branch().clone();
    let control = context.validation_input.control().clone();
    let retention_binding = context
        .validation_input
        .basis()
        .owner_inner()
        .retention_binding
        .clone();
    let decision_basis = context.validation_input.basis().clone();
    let coupled_proposal = context.validation_input.coupled_proposal().cloned();
    let expected_basis = decision_basis.descriptor().clone();
    let expected_root = std::sync::Arc::clone(&context.validation_input.basis().owner_inner().root);
    if let Some(interruption) =
        control.observe(contracts::runtime::operation_control::interruption_event::RelationalInterruptionBoundary::CandidatePreparation)
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
        },
    )
}

fn require_parent_publication_settled(
    history: &RelationalPreparationHistory,
    expected_root: &std::sync::Arc<
        worth_relational_branch::internal_port::branch::root::RelationalBranchRoot,
    >,
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
    TransactionCommitError::publication(
        contracts::publication::data::publication_error::PublicationError::new(
            contracts::publication::bundle::publication_status::PublicationStage::Visibility,
            detail,
        ),
    )
}
