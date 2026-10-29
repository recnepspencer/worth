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
    pub(crate) aggregate: Option<crate::aggregate_identity::AggregateProposal>,
    pub(crate) source_interpretation:
        std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
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

    pub(crate) fn aggregate_source<'a>(
        &'a self,
        selected: SelectedRelationalBranchRead<'a>,
    ) -> Option<crate::aggregate_identity::AggregateProposalBorrow<'a>> {
        self.aggregate
            .as_ref()
            .map(|owner| owner.borrow(&self.state, selected))
    }

    pub(crate) fn prepare_aggregate(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &RelationalPreparationRuntime,
        selected: SelectedRelationalBranchRead<'_>,
        business: &MergedCommitPlan,
    ) -> Result<(), MutationApplyDenial> {
        if let Some(owner) = &self.aggregate {
            let source = owner.borrow(&self.state, selected);
            source.ensure_prepared(execution, runtime.commit_work_budget.as_ref(), business)?;
            source.reserve_consequence_symbols(
                &mut self.identities.symbols,
                runtime.config.identity.client_key_symbol_policy,
                execution.allocation_policy(),
            )?;
        }
        Ok(())
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
    source_interpretation: std::sync::Arc<
        crate::validation::candidate_source::NativeManagedSourceInterpretation,
    >,
) -> Result<AppliedMutationDraft, MutationApplyDenial> {
    if !identity
        .source_interpretation()
        .is_some_and(|owner| std::sync::Arc::ptr_eq(&owner, &source_interpretation))
    {
        return Err(crate::transactions::data::TransactionCommitError::conflict(
            crate::transactions::data::CommitConflict::new(
                crate::transactions::data::ConflictClass::StaleValidationBasis {
                    detail: "candidate Source belongs to another proposal owner".into(),
                },
            ),
        )
        .into());
    }
    let materialization = source_interpretation.materialization_mode();
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
    crate::aggregate_identity::guard::ordinary_plan(
        execution,
        runtime.commit_work_budget.as_ref(),
        schema.registry(),
        &crate::validation::engine::state_view::InvariantStateView::new(
            selected_state,
            selected_state.version_id(),
        ),
        &merged_plan.merged_intents,
        materialization,
    )?;
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
    let aggregate = crate::aggregate_identity::AggregateProposal::capture(
        schema,
        selected_state.selected(),
        identity,
        outcome.effect.clone(),
        materialization,
    );
    Ok(AppliedMutationDraft {
        source_interpretation,
        aggregate,
        state,
        outcome,
        identities: PendingMutationIdentities { records, symbols },
    })
}
