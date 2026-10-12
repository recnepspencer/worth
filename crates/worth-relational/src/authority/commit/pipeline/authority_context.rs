use super::bulk_mutation_telemetry::{telemetry_from_strategy_batch, BulkMutationPlanTelemetry};
use super::rejection::invalid_merge_context;
use crate::commit_strategies::data::StrategyCommitArtifactBundle;
use crate::mvcc::RelationalTransactionValidationInput;
use crate::transactions::data::{
    CommitPhaseTiming, MergeCommitMutationPlan, MergedCommitPlan, TransactionCommitError,
    TransactionId,
};
use crate::validation::engine::InvariantExecutionResult;

#[derive(Debug)]
pub(super) enum CommitAuthorityInput {
    Mutation(MergedCommitPlan),
    Merge(MergeCommitMutationPlan),
}

#[derive(Debug)]
pub(crate) struct AuthoritativeCommitContext {
    pub(super) commit_work_budget: Option<crate::execution::RequestWorkBudget>,
    pub(super) _mutation_authority: Option<crate::branch::RelationalBranchMutationAuthority>,
    pub(super) transaction_id: TransactionId,
    pub(super) validation_input: RelationalTransactionValidationInput,
    pub(super) phase_timing: CommitPhaseTiming,
    pub(super) authority_input: CommitAuthorityInput,
    pub(super) prepared_scope: Option<PreparedAuthorityScope>,
    pub(super) merge_execution_accounting: Option<MergeExecutionAccounting>,
    pub(super) merge_execution_diagnostics_plan:
        Option<crate::merge::data::MergeExecutionDiagnosticsPlan>,
    pub(super) complexity_baseline: Option<crate::performance::data::RuntimeComplexityCounters>,
    pub(super) prior_complexity_delta: crate::performance::data::RuntimeComplexityCounters,
    pub(super) bulk_mutation_telemetry: Option<BulkMutationPlanTelemetry>,
    pub(super) prevalidated_commit_boundary: Option<InvariantExecutionResult>,
    pub(super) prevalidated_mutation_sensitive: Option<InvariantExecutionResult>,
    pub(super) prevalidated_snapshot_publication: Option<InvariantExecutionResult>,
    pub(super) validated_against_commit_id: Option<crate::history::data::CommitId>,
    pub(super) validated_against_version_id: Option<crate::identity::data::VersionId>,
    pub(super) validated_against_branch_version: Option<crate::branch::RelationalBranchVersion>,
    pub(super) strategy_commit_artifacts: Option<StrategyCommitArtifactBundle>,
}

#[derive(Debug)]
pub(super) struct PreparedAuthorityScope {
    pub(super) selected_branch_state: crate::branch::SelectedRelationalBranchState,
    pub(super) structural_summary:
        crate::authority::commit::structural_summary::CommitStructuralSummary,
    pub(super) working_state: crate::storage::overlay::WorkingState,
    pub(super) proposed_working_state: Option<crate::storage::overlay::WorkingState>,
    pub(super) proposal_identity: Option<crate::mvcc::RelationalMutationProposalIdentity>,
    pub(super) phase_timing: CommitPhaseTiming,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct MergeExecutionAccounting {
    pub(super) admitted_records: usize,
    pub(super) emitted_mutation_intents: usize,
}

impl AuthoritativeCommitContext {
    pub(crate) fn from_validated_proposal(
        candidate: crate::mvcc::ValidatedRelationalProposal,
    ) -> Self {
        let crate::mvcc::ValidatedRelationalProposal {
            mutation_authority,
            transaction_id,
            validation_input,
            prepared,
            proposed_working_state,
            commit_boundary,
            mutation_sensitive,
            snapshot_publication,
            proposal_identity,
            validated_against_commit,
            validated_against_version,
            validated_against_branch_version,
            batch_count,
            strategy_commit_artifacts,
            strategy_bulk_mutation_batch,
            validation_complexity_delta,
            commit_work_budget,
            ..
        } = candidate;
        let bulk_mutation_telemetry = strategy_bulk_mutation_batch
            .as_ref()
            .map(telemetry_from_strategy_batch)
            .or_else(|| {
                super::bulk_mutation_telemetry::summarize_bulk_mutation_telemetry(
                    &prepared.merged_plan,
                    batch_count,
                )
            });
        Self {
            commit_work_budget,
            _mutation_authority: Some(mutation_authority),
            transaction_id,
            validation_input,
            phase_timing: prepared.phase_timing.clone(),
            authority_input: CommitAuthorityInput::Mutation(prepared.merged_plan),
            prepared_scope: Some(PreparedAuthorityScope {
                selected_branch_state: prepared.selected_branch_state.clone(),
                structural_summary: prepared.structural_summary,
                working_state: prepared.working_state,
                proposed_working_state: Some(proposed_working_state),
                proposal_identity: Some(proposal_identity),
                phase_timing: prepared.phase_timing,
            }),
            merge_execution_accounting: None,
            merge_execution_diagnostics_plan: None,
            complexity_baseline: None,
            prior_complexity_delta: validation_complexity_delta,
            bulk_mutation_telemetry,
            prevalidated_commit_boundary: Some(commit_boundary),
            prevalidated_mutation_sensitive: Some(mutation_sensitive),
            prevalidated_snapshot_publication: Some(snapshot_publication),
            validated_against_commit_id: validated_against_commit,
            validated_against_version_id: Some(validated_against_version),
            validated_against_branch_version: Some(validated_against_branch_version),
            strategy_commit_artifacts,
        }
    }

    pub(crate) fn from_merge(
        validation_input: RelationalTransactionValidationInput,
        merge_plan: MergeCommitMutationPlan,
    ) -> Result<Self, TransactionCommitError> {
        Self::from_merge_execution(validation_input, merge_plan, None, None)
    }

    pub(crate) fn from_prepared_merge(
        validation_input: RelationalTransactionValidationInput,
        merge_plan: MergeCommitMutationPlan,
        diagnostics_plan: crate::merge::data::MergeExecutionDiagnosticsPlan,
        complexity_baseline: crate::performance::data::RuntimeComplexityCounters,
    ) -> Result<Self, TransactionCommitError> {
        Self::from_merge_execution(
            validation_input,
            merge_plan,
            Some(diagnostics_plan),
            Some(complexity_baseline),
        )
    }

    fn from_merge_execution(
        validation_input: RelationalTransactionValidationInput,
        merge_plan: MergeCommitMutationPlan,
        diagnostics_plan: Option<crate::merge::data::MergeExecutionDiagnosticsPlan>,
        complexity_baseline: Option<crate::performance::data::RuntimeComplexityCounters>,
    ) -> Result<Self, TransactionCommitError> {
        validate_merge_context_proof(&validation_input, &merge_plan)?;

        Ok(Self {
            commit_work_budget: None,
            _mutation_authority: None,
            transaction_id: merge_plan.transaction_id,
            validation_input,
            phase_timing: CommitPhaseTiming::default(),
            merge_execution_accounting: Some(MergeExecutionAccounting {
                admitted_records: merge_plan.structural_summary.executed_record_count,
                emitted_mutation_intents: merge_plan
                    .structural_summary
                    .emitted_mutation_intent_count,
            }),
            merge_execution_diagnostics_plan: diagnostics_plan,
            complexity_baseline,
            prior_complexity_delta: crate::performance::data::RuntimeComplexityCounters::default(),
            authority_input: CommitAuthorityInput::Merge(merge_plan),
            prepared_scope: None,
            bulk_mutation_telemetry: None,
            prevalidated_commit_boundary: None,
            prevalidated_mutation_sensitive: None,
            prevalidated_snapshot_publication: None,
            validated_against_commit_id: None,
            validated_against_version_id: None,
            validated_against_branch_version: None,
            strategy_commit_artifacts: None,
        })
    }
}

fn validate_merge_context_proof(
    validation_input: &RelationalTransactionValidationInput,
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
