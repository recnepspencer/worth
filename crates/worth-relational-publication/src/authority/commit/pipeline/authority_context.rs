use super::bulk_mutation_telemetry::{telemetry_from_strategy_batch, BulkMutationPlanTelemetry};
use super::publication_validation::DraftValidationInput;
use super::rejection::invalid_merge_context;
use crate::authority::commit::semantic_plan::PreparedRelationalCommitPlan;
use crate::transactions::data::outcomes::commit_result::CommitPhaseTiming;
use contracts::commit_strategies::data::artifacts::artifact_bundle::StrategyCommitArtifactBundle;
use contracts::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use contracts::transactions::data::primitives::TransactionId;
use mutation::validation::engine::result::InvariantExecutionResult;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;

#[derive(Debug)]
pub enum CommitAuthorityInput {
    Mutation(PreparedRelationalCommitPlan),
    Merge(mutation::mvcc::transaction::reconstructed_merge_plan::OwnedMergePlan),
}

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
    pub authority_input: CommitAuthorityInput,
    pub(super) prepared_scope: Option<PreparedAuthorityScope>,
    pub(super) merge_execution_accounting: Option<MergeExecutionAccounting>,
    pub(super) merge_execution_diagnostics_plan:
        Option<contracts::merge::data::execution_artifacts::MergeExecutionDiagnosticsPlan>,
    pub(super) complexity_baseline:
        Option<contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters>,
    pub(super) prior_complexity_delta:
        contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
    pub(super) bulk_mutation_telemetry: Option<BulkMutationPlanTelemetry>,
    pub(super) prevalidated_invariants: Option<InvariantExecutionResult>,
    pub(super) validated_against_commit_id: Option<contracts::history::data::coordinates::CommitId>,
    pub(super) validated_against_version_id: Option<contracts::identity::data::primitives::VersionId>,
    pub(super) validated_against_branch_version:
        Option<contracts::branch::version::RelationalBranchVersion>,
    pub(super) strategy_commit_artifacts: Option<StrategyCommitArtifactBundle>,
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

#[derive(Debug, Clone, Copy)]
pub(super) struct MergeExecutionAccounting {
    pub(super) admitted_records: usize,
    pub(super) emitted_mutation_intents: usize,
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
    pub strategy_commit_artifacts: Option<StrategyCommitArtifactBundle>,
    pub strategy_bulk_mutation_batch:
        Option<contracts::transactions::data::primitives::ProvenanceCompleteBulkMutationBatch>,
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
            strategy_commit_artifacts,
            strategy_bulk_mutation_batch,
            validation_complexity_delta,
            commit_work_budget,
        } = selected;
        let bulk_mutation_telemetry = strategy_bulk_mutation_batch
            .as_ref()
            .map(telemetry_from_strategy_batch)
            .or_else(|| {
                super::bulk_mutation_telemetry::summarize_bulk_mutation_telemetry(
                    prepared.plan.view().commands(),
                    batch_count,
                )
            });
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
            authority_input: CommitAuthorityInput::Mutation(prepared.plan),
            prepared_scope: Some(PreparedAuthorityScope {
                selected_branch_state: prepared.selected_branch_state.clone(),
                structural_summary: prepared.structural_summary,
                working_state: prepared.working_state,
                mutation_proposal: (proposal_identity, proposed_mutation),
                phase_timing: prepared.phase_timing,
            }),
            merge_execution_accounting: None,
            merge_execution_diagnostics_plan: None,
            complexity_baseline: None,
            prior_complexity_delta: validation_complexity_delta,
            bulk_mutation_telemetry,
            prevalidated_invariants: Some(invariants),
            validated_against_commit_id: validated_against_commit,
            validated_against_version_id: Some(validated_against_version),
            validated_against_branch_version: Some(validated_against_branch_version),
            strategy_commit_artifacts,
        }
    }

    pub fn from_merge(
        validation_input: V,
        merge_plan: mutation::mvcc::transaction::reconstructed_merge_plan::OwnedMergePlan,
    ) -> Result<Self, TransactionCommitError> {
        Self::from_merge_execution(validation_input, merge_plan, None, None)
    }

    pub fn from_prepared_merge(
        validation_input: V,
        merge_plan: MergeCommitMutationPlan,
        diagnostics_plan: contracts::merge::data::execution_artifacts::MergeExecutionDiagnosticsPlan,
        complexity_baseline: contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
    ) -> Result<Self, TransactionCommitError> {
        Self::from_merge_execution(
            validation_input,
            mutation::mvcc::transaction::reconstructed_merge_plan::OwnedMergePlan::from_ordinary_plan(
                merge_plan,
            ),
            Some(diagnostics_plan),
            Some(complexity_baseline),
        )
    }

    fn from_merge_execution(
        validation_input: V,
        merge_plan: mutation::mvcc::transaction::reconstructed_merge_plan::OwnedMergePlan,
        diagnostics_plan: Option<
            contracts::merge::data::execution_artifacts::MergeExecutionDiagnosticsPlan,
        >,
        complexity_baseline: Option<
            contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters,
        >,
    ) -> Result<Self, TransactionCommitError> {
        validate_merge_context_proof(&validation_input, merge_plan.plan())?;

        Ok(Self {
            recovered_index_catalog: None,
            caller_association: None,
            required_output_publication: None,
            required_output_materialization: None,
            commit_work_budget: None,
            _mutation_authority: None,
            transaction_id: merge_plan.plan().transaction_id,
            validation_input,
            phase_timing: CommitPhaseTiming::default(),
            merge_execution_accounting: Some(MergeExecutionAccounting {
                admitted_records: merge_plan.plan().structural_summary.executed_record_count,
                emitted_mutation_intents: merge_plan
                    .plan()
                    .structural_summary
                    .emitted_mutation_intent_count,
            }),
            merge_execution_diagnostics_plan: diagnostics_plan,
            complexity_baseline,
            prior_complexity_delta: contracts::performance::data::runtime_complexity_counters::RuntimeComplexityCounters::default(),
            authority_input: CommitAuthorityInput::Merge(merge_plan),
            prepared_scope: None,
            bulk_mutation_telemetry: None,
            prevalidated_invariants: None,
            validated_against_commit_id: None,
            validated_against_version_id: None,
            validated_against_branch_version: None,
            strategy_commit_artifacts: None,
        })
    }
}

fn validate_merge_context_proof(
    validation_input: &impl DraftValidationInput,
    merge_plan: &MergeCommitMutationPlan,
) -> Result<(), TransactionCommitError> {
    if validation_input.target_branch() != &merge_plan.target_branch {
        return Err(invalid_merge_context(
            "merge commit context target branch does not match merge proof",
        ));
    }
    if validation_input.merge_parent_branch_ids() != merge_plan.merge_parent_branches.as_ref() {
        return Err(invalid_merge_context(
            "merge commit context merge parent branches do not match merge proof",
        ));
    }
    if merge_plan.requested_merge_parent_count != merge_plan.merge_parent_branches.len() {
        return Err(invalid_merge_context(
            "merge commit context requested merge parent count does not match merge proof",
        ));
    }
    let expected_parents = [
        merge_plan.merge_execution_summary.target_head_commit_id,
        merge_plan.merge_execution_summary.source_head_commit_id,
    ];
    if merge_plan.parent_commits.as_slice() != expected_parents {
        return Err(invalid_merge_context(
            "merge commit context ordered parent commits do not match merge proof",
        ));
    }
    Ok(())
}
