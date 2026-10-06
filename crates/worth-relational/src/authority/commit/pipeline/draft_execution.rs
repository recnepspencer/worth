use super::authority_context::PreparedAuthorityScope;
use super::draft_preparation_phase::record_draft_preparation_phase;
use super::execution_admission::AdmittedCommitExecution;
use crate::authority::commit::phases::prepare::prepare_authoritative_working_state_scope_for_base;
use crate::authority::commit::phases::proposed_invariant_state::prepare_proposed_invariant_state;

pub(super) struct PreparedCommitExecution {
    admitted: AdmittedCommitExecution,
    public_structural_summary: crate::transactions::data::CommitStructuralSummary,
    working_state: crate::storage::overlay::WorkingState,
    proposed_working_state: crate::storage::overlay::WorkingState,
    proposed_version_id: crate::identity::data::VersionId,
    proposal_identity: crate::mvcc::RelationalMutationProposalIdentity,
}

impl PreparedCommitExecution {
    pub(super) fn admitted_mut(&mut self) -> &mut AdmittedCommitExecution {
        &mut self.admitted
    }

    pub(super) fn selected_branch_state(&self) -> &crate::branch::SelectedRelationalBranchState {
        self.admitted.selected_branch_state()
    }

    pub(super) fn proposed_version_id(&self) -> crate::identity::data::VersionId {
        self.proposed_version_id
    }

    pub(super) fn proposal_identity(&self) -> &crate::mvcc::RelationalMutationProposalIdentity {
        &self.proposal_identity
    }

    pub(super) fn mutation_parts(
        &mut self,
    ) -> (
        &mut AdmittedCommitExecution,
        &mut crate::storage::overlay::WorkingState,
    ) {
        (&mut self.admitted, &mut self.working_state)
    }

    pub(super) fn boundary_parts(
        &mut self,
    ) -> (
        &mut AdmittedCommitExecution,
        &crate::storage::overlay::WorkingState,
        crate::identity::data::VersionId,
        &crate::mvcc::RelationalMutationProposalIdentity,
    ) {
        (
            &mut self.admitted,
            &self.proposed_working_state,
            self.proposed_version_id,
            &self.proposal_identity,
        )
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        AdmittedCommitExecution,
        crate::transactions::data::CommitStructuralSummary,
        crate::storage::overlay::WorkingState,
    ) {
        let Self {
            admitted,
            public_structural_summary,
            working_state,
            ..
        } = self;
        (admitted, public_structural_summary, working_state)
    }
}

pub(super) fn prepare_commit_execution(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut admitted: AdmittedCommitExecution,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<PreparedCommitExecution, crate::transactions::data::TransactionCommitError> {
    let merge_parent_count = admitted
        .merge_history_plan()
        .map(|plan| plan.requested_merge_parent_count)
        .unwrap_or(admitted.validation_input().merge_parent_bases().len());
    let PreparedAuthorityScope {
        structural_summary,
        working_state,
        proposed_working_state,
        proposal_identity,
        phase_timing: prepared_timing,
        ..
    } = match admitted.take_prepared_scope() {
        Some(scope) => scope,
        None => {
            let selected_branch_state = admitted.selected_branch_state();
            let (structural_summary, working_state, phase_timing) =
                prepare_authoritative_working_state_scope_for_base(
                    runtime,
                    selected_branch_state.state(),
                    admitted.merged_plan(),
                    merge_parent_count,
                    Some(admitted.validation_input().footprint()),
                );
            PreparedAuthorityScope {
                selected_branch_state: selected_branch_state.clone(),
                structural_summary,
                working_state,
                proposed_working_state: None,
                proposal_identity: None,
                phase_timing,
            }
        }
    };
    let proposal_identity = match proposal_identity {
        Some(identity) => identity,
        None => runtime.issue_mutation_proposal_identity(
            admitted.transaction_id(),
            admitted.validation_input(),
        )?,
    };
    let proposed_version_id = proposal_identity.proposed_version_id();
    let proposed_working_state = match proposed_working_state {
        Some(proposed) => proposed,
        None => prepare_proposed_invariant_state(
            runtime,
            admitted.selected_branch_state(),
            &working_state,
            admitted.merged_plan(),
            admitted.validation_input().schema_authority(),
            proposed_version_id,
            lease,
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
        admitted,
        public_structural_summary,
        working_state,
        proposed_working_state,
        proposed_version_id,
        proposal_identity,
    })
}
