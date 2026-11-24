//! The exact applied draft judged by invariants and later offered for publication.
use crate::authority::commit::phases::mutation::branch_local_delete_allowance_for_plan;
use crate::authority::mutation::{
    apply_plan_with_publication_edges, MutationApplyDenial, MutationApplyOutcome,
};
use crate::branch::selected_read::SelectedRelationalBranchRead;
use crate::runtime::state::subsystems::services::symbol_table::PendingSymbolMeanings;

use crate::storage::overlay::WorkingState;
use crate::transactions::data::outcomes::plan_artifacts::AuthoritativeApplyPlan;
use crate::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;

pub(crate) use crate::authority::mutation::pending_identities::PendingMutationIdentities;

mod aggregate_consequences;
pub(crate) mod inputs;
use inputs::AppliedMutationInputs;
pub(crate) mod aggregate_inputs;
mod aggregate_metrics;
mod coupled_return;
pub(crate) use coupled_return::{
    AppliedCoreCustody, AppliedDraftCustody, CoupledPreparationCleanup,
};
pub(crate) mod content;
pub(crate) mod ordinary_apply;
pub(crate) mod owned_graph_selection;
pub(crate) use ordinary_apply::AppliedMutationCore;

pub(crate) struct AppliedMutationDraft {
    pub(crate) state: WorkingState,
    pub(crate) content: crate::content::publication::ContentPublication,
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

    pub(crate) fn prepare_aggregate<'a>(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        inputs: impl Into<AppliedMutationInputs<'a>>,
        selected: SelectedRelationalBranchRead<'_>,
        business: &MergedCommitPlan,
    ) -> Result<(), MutationApplyDenial> {
        let inputs = inputs.into();
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
            source.ensure_prepared(execution, inputs.work.as_ref(), business)?;
            source.reserve_consequence_symbols(
                &mut self.identities.symbols,
                inputs.key_policy,
                execution.allocation_policy(),
            )?;
        }
        Ok(())
    }

    pub(crate) fn candidate_symbols(&self) -> &PendingSymbolMeanings {
        &self.identities.symbols
    }
}

pub(crate) fn apply_mutation_draft<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<AppliedMutationInputs<'a>>,
    selected_state: &SelectedRelationalBranchRead<'_>,
    working_state: &WorkingState,
    merged_plan: &MergedCommitPlan,
    schema: &crate::branch::root::schema::RelationalBranchRootSchemaAuthority,
    identity: &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    mut symbols: PendingSymbolMeanings,
    source_interpretation: std::sync::Arc<
        crate::validation::candidate_source::NativeManagedSourceInterpretation,
    >,
    content: &crate::content::RelationalContentAttachments,
) -> Result<AppliedDraftCustody, MutationApplyDenial> {
    let inputs = inputs.into();
    require_original_source_interpretation(identity, &source_interpretation)?;
    let materialization = source_interpretation.materialization_mode();
    let version_id = identity.proposed_version_id();
    let _apply_memory = crate::mvcc::transaction::normalized_plan::reserve_copy(
        execution,
        crate::mvcc::transaction::normalized_plan::intent_bytes(&merged_plan.merged_intents),
    )?;
    let apply_plan = AuthoritativeApplyPlan {
        transaction_id: merged_plan.transaction_id,
        version_id,
        merged_intents: merged_plan.merged_intents.clone(),
    };
    let config = inputs.mutation_config();
    let mut core = if let Some(binding) = inputs.authored_coupling {
        binding.adopt_applied(execution, identity, merged_plan, &mut symbols)?
    } else {
        AppliedCoreCustody::ordinary(ordinary_apply::apply(
            execution,
            inputs,
            selected_state,
            working_state,
            merged_plan,
            &apply_plan,
            &config,
            schema,
            identity,
            symbols,
            materialization,
        )?)
    };
    core.content = content::prepare(
        execution,
        inputs,
        selected_state.selected().state().root(),
        schema,
        content,
        &core.state,
        &core.outcome.effect.publication.changed_records,
        version_id,
        source_interpretation.restoration_content(),
    )?;
    source_interpretation
        .bind_content(
            execution,
            core.content.directory().map_err(|_| {
                crate::content::publication::commit_denial(
                    crate::content::RelationalContentDenial::EncodingInconsistency,
                )
            })?,
            schema,
        )
        .map_err(crate::content::publication::commit_denial)?;
    Ok(core.into_draft(source_interpretation))
}

pub(crate) fn apply_retained_entity_selection_draft<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<AppliedMutationInputs<'a>>,
    selected_state: &SelectedRelationalBranchRead<'_>,
    working_state: &WorkingState,
    retained: &crate::authority::commit::semantic_plan::PreparedRetainedEntityPayload,
    schema: &crate::branch::root::schema::RelationalBranchRootSchemaAuthority,
    identity: &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    mut symbols: PendingSymbolMeanings,
    source_interpretation: std::sync::Arc<
        crate::validation::candidate_source::NativeManagedSourceInterpretation,
    >,
    content: &crate::content::RelationalContentAttachments,
) -> Result<AppliedMutationDraft, MutationApplyDenial> {
    let inputs = inputs.into();
    require_original_source_interpretation(identity, &source_interpretation)?;
    crate::aggregate_identity::guard::retained_entity_selection(
        execution,
        inputs.work.as_ref(),
        schema.registry(),
        retained.meaning.kind(),
    )?;
    let mut state = working_state.clone();
    let mut records = inputs.record_identity.begin_allocations();
    let config = inputs.mutation_config();
    let outcome = crate::execution::transaction_stage(execution, inputs.work.as_ref(), || {
        crate::authority::mutation::retained_entity_selection::apply(
            execution,
            &mut state,
            retained,
            identity.proposed_version_id(),
            &config,
            schema.registry(),
            schema.aspect_plans(),
            &mut symbols,
            &mut records,
            inputs.work.clone(),
        )
    })?;
    let prepared_content = content::prepare(
        execution,
        inputs,
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
            prepared_content.directory().map_err(|_| {
                crate::content::publication::commit_denial(
                    crate::content::RelationalContentDenial::EncodingInconsistency,
                )
            })?,
            schema,
        )
        .map_err(crate::content::publication::commit_denial)?;
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
        content: prepared_content,
        source_interpretation,
        state,
        outcome,
        aggregate,
        identities: PendingMutationIdentities { records, symbols },
    })
}

fn require_original_source_interpretation(
    identity: &crate::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    source: &std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
) -> Result<(), MutationApplyDenial> {
    if identity
        .source_interpretation()
        .is_some_and(|owner| std::sync::Arc::ptr_eq(&owner, source))
    {
        return Ok(());
    }
    Err(crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(
        crate::transactions::data::outcomes::commit_conflict::CommitConflict::new(
            crate::transactions::data::outcomes::conflict_class::ConflictClass::StaleValidationBasis {
                detail: "candidate Source belongs to another proposal owner".into(),
            },
        ),
    )
    .into())
}
