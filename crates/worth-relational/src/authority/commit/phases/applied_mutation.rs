//! The exact applied draft judged by invariants and later offered for publication.
use crate::authority::commit::phases::mutation::branch_local_delete_allowance_for_plan;
use crate::authority::mutation::{
    apply_plan_with_publication_edges, MutationApplyDenial, MutationApplyOutcome,
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

mod aggregate_consequences;
pub(crate) mod content;
pub(crate) mod ordinary_apply;
pub(crate) mod owned_graph_selection;
pub(crate) use ordinary_apply::AppliedMutationCore;

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
        crate::publication::ownership::reserve_symbols(
            selected
                .selected()
                .state()
                .root()
                .retained_schema_authority()
                .registry(),
            &mut self.identities.symbols,
        )?;
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
    content: &crate::content::RelationalContentAttachments,
) -> Result<AppliedMutationDraft, MutationApplyDenial> {
    require_original_source_interpretation(identity, &source_interpretation)?;
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
    let core = if let Some(binding) = &runtime.authored_coupling {
        binding.adopt_applied(execution, identity, merged_plan, &mut symbols)?
    } else {
        ordinary_apply::apply(
            execution,
            runtime,
            selected_state,
            working_state,
            merged_plan,
            &apply_plan,
            &config,
            schema,
            identity,
            symbols,
            materialization,
        )?
    };
    let AppliedMutationCore {
        mut state,
        outcome,
        aggregate,
        identities: PendingMutationIdentities { records, symbols },
        coupled_publication: _,
    } = core;
    state.content = content::prepare(
        execution,
        runtime,
        selected_state.selected().state().root(),
        schema,
        content,
        &state,
        &outcome.effect.publication.changed_records,
        version_id,
        source_interpretation.restoration_content(),
    )?;
    source_interpretation
        .bind_content(
            execution,
            state.content.directory().map_err(|_| {
                crate::content::commit_denial(
                    crate::content::RelationalContentDenial::EncodingInconsistency,
                )
            })?,
            schema,
        )
        .map_err(crate::content::commit_denial)?;
    Ok(AppliedMutationDraft {
        source_interpretation,
        aggregate,
        state,
        outcome,
        identities: PendingMutationIdentities { records, symbols },
    })
}

pub(crate) fn apply_retained_entity_selection_draft(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    selected_state: &SelectedRelationalBranchRead<'_>,
    working_state: &WorkingState,
    retained: &crate::authority::commit::semantic_plan::PreparedRetainedEntityPayload,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    identity: &crate::mvcc::RelationalMutationProposalIdentity,
    mut symbols: PendingSymbolMeanings,
    source_interpretation: std::sync::Arc<
        crate::validation::candidate_source::NativeManagedSourceInterpretation,
    >,
    content: &crate::content::RelationalContentAttachments,
) -> Result<AppliedMutationDraft, MutationApplyDenial> {
    require_original_source_interpretation(identity, &source_interpretation)?;
    crate::aggregate_identity::guard::retained_entity_selection(
        execution,
        runtime.commit_work_budget.as_ref(),
        schema.registry(),
        retained.meaning.kind(),
    )?;
    let mut state = working_state.clone();
    let mut records = runtime.record_identity.begin_allocations();
    let config = crate::config::data::MutationConfig {
        cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
        adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
        cross_context_policy: runtime.config.storage.cross_context_policy,
    };
    let outcome = crate::authority::mutation::retained_entity_selection::apply(
        &mut state,
        retained,
        identity.proposed_version_id(),
        &config,
        schema.registry(),
        schema.aspect_plans(),
        &mut symbols,
        &mut records,
        runtime.commit_work_budget.clone(),
    )?;
    state.content = content::prepare(
        execution,
        runtime,
        selected_state.selected().state().root(),
        schema,
        content,
        &state,
        &outcome.effect.publication.changed_records,
        identity.proposed_version_id(),
        Some(retained._source.inner.root.retained_content()),
    )?;
    source_interpretation
        .bind_content(
            execution,
            state.content.directory().map_err(|_| {
                crate::content::commit_denial(
                    crate::content::RelationalContentDenial::EncodingInconsistency,
                )
            })?,
            schema,
        )
        .map_err(crate::content::commit_denial)?;
    let aggregate = crate::aggregate_identity::AggregateProposal::capture(
        schema,
        selected_state.selected(),
        identity,
        &outcome.effect.publication.changed_records,
        &outcome.effect.adjacency.deltas,
        outcome.effect.publication.has_entity_deletions,
        None,
    );
    Ok(AppliedMutationDraft {
        source_interpretation,
        state,
        outcome,
        aggregate,
        identities: PendingMutationIdentities { records, symbols },
    })
}

fn require_original_source_interpretation(
    identity: &crate::mvcc::RelationalMutationProposalIdentity,
    source: &std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
) -> Result<(), MutationApplyDenial> {
    if identity
        .source_interpretation()
        .is_some_and(|owner| std::sync::Arc::ptr_eq(&owner, source))
    {
        return Ok(());
    }
    Err(crate::transactions::data::TransactionCommitError::conflict(
        crate::transactions::data::CommitConflict::new(
            crate::transactions::data::ConflictClass::StaleValidationBasis {
                detail: "candidate Source belongs to another proposal owner".into(),
            },
        ),
    )
    .into())
}
