use super::authority_context::PreparedAuthorityScope;
use super::draft_preparation_phase::record_draft_preparation_phase;
use super::execution_admission::{AdmittedCommitExecution, AdmittedCommitState};
use super::phase_backing::CommitPhaseBacking;
use super::publication_validation::DraftValidationInput;
use crate::authority::commit::phases::applied_mutation::AppliedMutationDraft;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_mutation::internal_port as mutation;

pub(super) struct PreparedCommitState {
    pub(super) admitted: AdmittedCommitState,
    pub(super) public_structural_summary:
        contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
    pub(super) proposal_identity:
        mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
}

impl PreparedCommitState {
    pub(super) fn selected_branch_state(
        &self,
    ) -> &worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState{
        self.admitted.selected_branch_state()
    }
    pub(super) fn version_id(&self) -> contracts::identity::data::primitives::VersionId {
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
        &mut AdmittedCommitState,
        &worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
        contracts::identity::data::primitives::VersionId,
        &mutation::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
        Option<mutation::aggregate_identity::proposal::AggregateProposalBorrow<'_>>,
    ) {
        (
            &mut self.state.admitted,
            self.mutation.state(),
            self.state.proposal_identity.proposed_version_id(),
            &self.state.proposal_identity,
            self.mutation
                .aggregate
                .as_ref()
                .map(|owner| owner.borrow_retained(self.mutation.state())),
        )
    }

    pub(super) fn into_parts(self) -> (PreparedCommitState, AppliedMutationDraft) {
        (self.state, self.mutation)
    }
}

pub(super) fn prepare_commit_execution<V: DraftValidationInput>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    instrumentation: &worth_relational_branch::internal_port::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    descriptor_semantics: contracts::schema::data::continuity::descriptor_versions::DescriptorSemanticsVersion,
    merge_draft: impl FnOnce(
        &worth_execution::ExecutionRequest<'_, '_>,
        &AdmittedCommitState<V>,
    ) -> Result<
        super::merge_draft::AcceptedMergeDraft,
        contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
    >,
    admitted: AdmittedCommitExecution<V>,
) -> Result<
    CommitPhaseBacking<PreparedCommitExecution>,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    // Producer temporaries retire before funding the retained successor,
    // matching the former caller-side allocation boundary.
    let result = {
        super::consult_stage(execution, super::CommitStage::Draft)?;
        let (mut admitted, prepared_scope) = admitted.into_parts();
        let PreparedAuthorityScope {
            structural_summary,
            working_state,
            mutation_proposal: (proposal_identity, mutation),
            phase_timing: prepared_timing,
            ..
        } = match prepared_scope {
            Some(scope) => scope,
            None => {
                let draft = merge_draft(execution, &admitted)?;
                admitted.accept_merge_draft(draft)
            }
        };
        admitted.phase_timing_mut().draft_structural_summary_micros =
            prepared_timing.draft_structural_summary_micros;
        admitted.phase_timing_mut().draft_working_state_clone_micros =
            prepared_timing.draft_working_state_clone_micros;
        let public_structural_summary = structural_summary.public_summary(descriptor_semantics);
        let (commit_log, phase_timing) = admitted.commit_phase_state();
        record_draft_preparation_phase(
            instrumentation,
            commit_log,
            phase_timing,
            &working_state,
            &structural_summary,
            &public_structural_summary,
        );
        PreparedCommitExecution {
            state: PreparedCommitState {
                admitted: admitted.into_publication(),
                public_structural_summary,
                proposal_identity,
            },
            mutation,
        }
    };
    CommitPhaseBacking::new(execution, result)
}
#[cfg(test)]
#[path = "draft_execution_inline_type_budget.rs"]
pub mod inline_type_budget;
