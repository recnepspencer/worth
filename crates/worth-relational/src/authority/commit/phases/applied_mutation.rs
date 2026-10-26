//! The exact applied draft judged by invariants and later offered for publication.
use crate::authority::commit::phases::mutation::branch_local_delete_allowance_for_plan;
use crate::authority::mutation::{
    apply_plan_to_working_state, MutationApplyDenial, MutationApplyOutcome,
};
use crate::branch::SelectedRelationalBranchRead;
use crate::runtime::{
    PendingRecordAllocations, PendingSymbolMeanings, RelationalPreparationRuntime, WorkingState,
};
use crate::transactions::data::{AuthoritativeApplyPlan, MergedCommitPlan};

#[derive(Debug)]
pub(crate) struct PendingMutationIdentities {
    pub(crate) records: PendingRecordAllocations,
    pub(crate) symbols: PendingSymbolMeanings,
}

pub(crate) struct AppliedMutationDraft {
    pub(crate) state: WorkingState,
    pub(crate) outcome: MutationApplyOutcome,
    pub(crate) identities: PendingMutationIdentities,
}

impl std::fmt::Debug for AppliedMutationDraft {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AppliedMutationDraft")
            .field("identities", &self.identities)
            .finish_non_exhaustive()
    }
}

impl AppliedMutationDraft {
    pub(crate) fn state(&self) -> &WorkingState {
        &self.state
    }

    pub(crate) fn candidate_symbols(&self) -> &PendingSymbolMeanings {
        &self.identities.symbols
    }
}

pub(crate) fn apply_mutation_draft(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    selected_state: &SelectedRelationalBranchRead<'_>,
    working_state: &WorkingState,
    merged_plan: &MergedCommitPlan,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    identity: &crate::mvcc::RelationalMutationProposalIdentity,
    mut symbols: PendingSymbolMeanings,
) -> Result<AppliedMutationDraft, MutationApplyDenial> {
    let version_id = identity.proposed_version_id();
    let apply_plan = AuthoritativeApplyPlan {
        transaction_id: merged_plan.transaction_id,
        version_id,
        merged_intents: merged_plan.merged_intents.clone(),
    };
    let config = crate::config::data::MutationConfig {
        cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
        adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
        cross_context_policy: runtime.config.storage.cross_context_policy,
    };
    let mut state = working_state.clone();
    let allowance = branch_local_delete_allowance_for_plan(selected_state, &state, merged_plan);
    let mut records = runtime.record_identity.begin_allocations();
    let outcome = apply_plan_to_working_state(
        &mut state,
        &apply_plan,
        &config,
        schema.registry(),
        schema.aspect_plans(),
        &mut symbols,
        allowance,
        &mut records,
        crate::authority::commit::pipeline::request_lease(execution),
        runtime.commit_work_budget.clone(),
    )?;
    crate::authority::mutation::apply_adjacency_deltas(
        &mut state,
        &outcome.effect.adjacency.deltas,
        version_id,
    );
    Ok(AppliedMutationDraft {
        state,
        outcome,
        identities: PendingMutationIdentities { records, symbols },
    })
}
