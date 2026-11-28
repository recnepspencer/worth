use super::bulk_mutation_telemetry::BulkMutationPlanTelemetry;
use super::publication_validation::DraftValidationInput;
use crate::authority::commit::semantic_plan::PreparedRelationalCommitPlan;
use crate::transactions::data::outcomes::commit_result::CommitPhaseTiming;
use contracts::transactions::data::primitives::TransactionId;
use mutation::validation::engine::result::InvariantExecutionResult;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;

#[derive(Debug)]
pub struct AuthoritativeCommitContext<V> {
    pub recovered_index_catalog: Option<worth_relational_durability::internal_port::durability::authority::runtime_rebuild::index_catalog::RecoveredIndexCatalog>,
    pub caller_association: Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>,
    pub(super) required_output_publication: Option<
        crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission,
    >,
    pub(super) required_output_materialization:
        Option<std::sync::Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_materialization::CanonicalRequiredOutputMaterialization>>,
    pub commit_work_budget: Option<contracts::execution::request_work::RequestWorkBudget>,
    pub(super) _mutation_authority:
        Option<worth_relational_branch::internal_port::branch::authority::RelationalBranchMutationAuthority>,
    pub(super) transaction_id: TransactionId,
    pub validation_input: V,
    pub(super) phase_timing: CommitPhaseTiming,
    pub authority_input: PreparedRelationalCommitPlan,
    pub(super) prepared_scope: PreparedAuthorityScope,
    pub(super) prior_complexity_delta:
        contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
    pub(super) bulk_mutation_telemetry: Option<BulkMutationPlanTelemetry>,
    pub(super) prevalidated_invariants: Option<InvariantExecutionResult>,
    pub(super) validated_against_commit_id: Option<contracts::history::data::coordinates::CommitId>,
    pub(super) validated_against_version_id: Option<contracts::identity::data::primitives::VersionId>,
    pub(super) validated_against_branch_version:
        Option<contracts::branch::version::RelationalBranchVersion>,
}

#[derive(Debug)]
pub struct PreparedAuthorityScope {
    pub selected_branch_state: worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    pub structural_summary:
        crate::authority::commit::structural_summary::CommitStructuralSummary,
    pub working_state: worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    pub mutation_proposal: (
        mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
        crate::authority::commit::phases::applied_mutation::AppliedMutationDraft,
    ),
    pub phase_timing: CommitPhaseTiming,
}

/// The original admitted operands consumed by one publication round.
pub struct SelectedCommitMutation<V> {
    pub caller_association: Option<worth_relational_branch::internal_port::history::data::caller_association::RelationalCallerAssociation>,
    pub required_output_publication: Option<
        crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission,
    >,
    pub required_output_materialization:
        Option<std::sync::Arc<worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_materialization::CanonicalRequiredOutputMaterialization>>,
    pub commit_work_budget: Option<contracts::execution::request_work::RequestWorkBudget>,
    pub mutation_authority: worth_relational_branch::internal_port::branch::authority::RelationalBranchMutationAuthority,
    pub transaction_id: TransactionId,
    pub validation_input: V,
    pub prepared: crate::authority::commit::phases::prepare::PreparedWorkingStateScope,
    pub proposed_mutation:
        crate::authority::commit::phases::applied_mutation::AppliedMutationDraft,
    pub invariants: InvariantExecutionResult,
    pub proposal_identity:
        mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    pub validated_against_commit: Option<contracts::history::data::coordinates::CommitId>,
    pub validated_against_version: contracts::identity::data::primitives::VersionId,
    pub validated_against_branch_version: contracts::branch::version::RelationalBranchVersion,
    pub batch_count: usize,
    pub validation_complexity_delta:
        contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
}

impl<V: DraftValidationInput> AuthoritativeCommitContext<V> {
    pub fn with_recovered_index_catalog(
        mut self,
        catalog: worth_relational_durability::internal_port::durability::authority::runtime_rebuild::index_catalog::RecoveredIndexCatalog,
    ) -> Self {
        self.recovered_index_catalog = Some(catalog);
        self
    }

    pub fn from_validated_proposal(candidate: impl Into<Self>) -> Self {
        candidate.into()
    }

    pub fn from_selected_mutation(selected: SelectedCommitMutation<V>) -> Self {
        let SelectedCommitMutation {
            caller_association,
            required_output_publication,
            required_output_materialization,
            mutation_authority,
            transaction_id,
            validation_input,
            prepared,
            proposed_mutation,
            invariants,
            proposal_identity,
            validated_against_commit,
            validated_against_version,
            validated_against_branch_version,
            batch_count,
            validation_complexity_delta,
            commit_work_budget,
        } = selected;
        let bulk_mutation_telemetry =
            super::bulk_mutation_telemetry::summarize_bulk_mutation_telemetry(
                prepared.plan.view().commands(),
                batch_count,
            );
        Self {
            recovered_index_catalog: None,
            caller_association,
            required_output_publication,
            required_output_materialization,
            commit_work_budget,
            _mutation_authority: Some(mutation_authority),
            transaction_id,
            validation_input,
            phase_timing: prepared.phase_timing.clone(),
            authority_input: prepared.plan,
            prepared_scope: PreparedAuthorityScope {
                selected_branch_state: prepared.selected_branch_state.clone(),
                structural_summary: prepared.structural_summary,
                working_state: prepared.working_state,
                mutation_proposal: (proposal_identity, proposed_mutation),
                phase_timing: prepared.phase_timing,
            },
            prior_complexity_delta: validation_complexity_delta,
            bulk_mutation_telemetry,
            prevalidated_invariants: Some(invariants),
            validated_against_commit_id: validated_against_commit,
            validated_against_version_id: Some(validated_against_version),
            validated_against_branch_version: Some(validated_against_branch_version),
        }
    }
}
