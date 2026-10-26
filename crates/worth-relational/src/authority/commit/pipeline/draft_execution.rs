use super::authority_context::PreparedAuthorityScope;
use super::draft_preparation_phase::record_draft_preparation_phase;
use super::execution_admission::AdmittedCommitExecution;
use crate::authority::commit::phases::applied_mutation::{
    apply_mutation_draft, AppliedMutationDraft,
};
use crate::authority::commit::phases::prepare::prepare_authoritative_working_state_scope_for_base;

pub(super) struct PreparedCommitState {
    pub(super) admitted: AdmittedCommitExecution,
    pub(super) public_structural_summary: crate::transactions::data::CommitStructuralSummary,
    pub(super) proposal_identity: crate::mvcc::RelationalMutationProposalIdentity,
}

impl PreparedCommitState {
    pub(super) fn selected_branch_state(&self) -> &crate::branch::SelectedRelationalBranchState {
        self.admitted.selected_branch_state()
    }
    pub(super) fn version_id(&self) -> crate::identity::data::VersionId {
        self.proposal_identity.proposed_version_id()
    }
}

pub(super) struct PreparedCommitExecution {
    state: PreparedCommitState,
    mutation: AppliedMutationDraft,
}

impl PreparedCommitExecution {
    pub(super) fn boundary_parts(
        &mut self,
    ) -> (
        &mut AdmittedCommitExecution,
        &crate::storage::overlay::WorkingState,
        crate::identity::data::VersionId,
        &crate::mvcc::RelationalMutationProposalIdentity,
    ) {
        (
            &mut self.state.admitted,
            self.mutation.state(),
            self.state.proposal_identity.proposed_version_id(),
            &self.state.proposal_identity,
        )
    }

    pub(super) fn candidate_symbols(&self) -> &crate::runtime::PendingSymbolMeanings {
        self.mutation.candidate_symbols()
    }

    pub(super) fn into_parts(self) -> (PreparedCommitState, AppliedMutationDraft) {
        (self.state, self.mutation)
    }
}

pub(super) fn prepare_commit_execution(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut admitted: AdmittedCommitExecution,
) -> Result<PreparedCommitExecution, crate::transactions::data::TransactionCommitError> {
    super::consult_stage(execution, super::CommitStage::Draft)?;
    let merge_parent_count = admitted
        .merge_history_plan()
        .map(|plan| plan.requested_merge_parent_count)
        .unwrap_or(admitted.validation_input().merge_parent_bases().len());
    let PreparedAuthorityScope {
        structural_summary,
        working_state,
        mutation_proposal,
        phase_timing: prepared_timing,
        ..
    } = match admitted.take_prepared_scope() {
        Some(scope) => scope,
        None => {
            let selected = admitted.selected_branch_state();
            let (structural_summary, working_state, phase_timing) =
                prepare_authoritative_working_state_scope_for_base(
                    runtime,
                    selected.state(),
                    admitted.merged_plan(),
                    merge_parent_count,
                    Some(admitted.validation_input().footprint()),
                );
            PreparedAuthorityScope {
                selected_branch_state: selected.clone(),
                structural_summary,
                working_state,
                mutation_proposal: None,
                phase_timing,
            }
        }
    };
    let (proposal_identity, mutation) = match mutation_proposal {
        Some(proposal) => proposal,
        None => crate::authority::commit::preparation::probe_symbol_rounds(
            execution,
            runtime,
            admitted.validation_input().basis(),
            |symbols, round| {
                let identity = runtime.issue_mutation_proposal_identity(
                    admitted.transaction_id(),
                    admitted.validation_input(),
                )?;
                let mutation = apply_mutation_draft(
                    execution,
                    round,
                    &crate::branch::SelectedRelationalBranchRead::in_round(
                        admitted.selected_branch_state(),
                        admitted.selected_branch_state(),
                    ),
                    &working_state,
                    admitted.merged_plan(),
                    admitted.validation_input().schema_authority(),
                    &identity,
                    symbols,
                )?;
                Ok((identity, mutation))
            },
        )?,
    };
    admitted.phase_timing_mut().draft_structural_summary_micros =
        prepared_timing.draft_structural_summary_micros;
    admitted.phase_timing_mut().draft_working_state_clone_micros =
        prepared_timing.draft_working_state_clone_micros;
    let public_structural_summary = structural_summary.public_summary(
        runtime
            .config
            .schema
            .descriptor_semantics_policy
            .current_write_version(),
    );
    let (commit_log, phase_timing) = admitted.commit_phase_state();
    record_draft_preparation_phase(
        runtime,
        commit_log,
        phase_timing,
        &working_state,
        &structural_summary,
        &public_structural_summary,
    );
    Ok(PreparedCommitExecution {
        state: PreparedCommitState {
            admitted,
            public_structural_summary,
            proposal_identity,
        },
        mutation,
    })
}
