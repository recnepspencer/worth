//! One ordinary mutation application owner, shared by a complete ordinary plan
//! and the authored prefix of a coupled producer assembly.
use super::*;

pub(crate) struct AppliedMutationCore {
    pub(crate) state: WorkingState,
    pub(crate) outcome: MutationApplyOutcome,
    pub(crate) identities: PendingMutationIdentities,
    pub(crate) aggregate: Option<crate::aggregate_identity::AggregateProposal>,
    pub(crate) coupled_publication: Option<crate::mvcc::RequiredOutputPublicationAdmission>,
}

impl AppliedMutationCore {
    pub(crate) fn state(&self) -> &WorkingState {
        &self.state
    }
    pub(crate) fn candidate_symbols(&self) -> &PendingSymbolMeanings {
        &self.identities.symbols
    }
}

pub(crate) fn apply(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    selected_state: &SelectedRelationalBranchRead<'_>,
    working_state: &WorkingState,
    merged_plan: &MergedCommitPlan,
    apply_plan: &AuthoritativeApplyPlan,
    config: &crate::config::data::MutationConfig,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    identity: &crate::mvcc::RelationalMutationProposalIdentity,
    mut symbols: PendingSymbolMeanings,
    materialization: Option<crate::mvcc::RelationalMaterializationTransactionMode>,
) -> Result<AppliedMutationCore, MutationApplyDenial> {
    let version_id = identity.proposed_version_id();
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
    let outcome = apply_plan_with_publication_edges(
        &mut state,
        apply_plan,
        config,
        schema.registry(),
        schema.aspect_plans(),
        &mut symbols,
        allowance,
        &mut records,
        crate::authority::commit::pipeline::request_lease(execution),
        runtime.commit_work_budget.clone(),
        runtime
            .authored_coupling
            .as_ref()
            .map(|binding| &binding.owners.deferred_edges),
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
        &outcome.effect.publication.changed_records,
        &outcome.effect.adjacency.deltas,
        outcome.effect.publication.has_entity_deletions,
        materialization,
    );
    Ok(AppliedMutationCore {
        aggregate,
        state,
        outcome,
        identities: PendingMutationIdentities { records, symbols },
        coupled_publication: None,
    })
}

/// The authored prefix uses the same mutation owner and the enclosing round's
/// actual Native proposal identity, allocation stream and symbol reservations.
pub(crate) fn apply_authored_prefix(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    selected: &SelectedRelationalBranchRead<'_>,
    prepared: &super::super::prepare::PreparedWorkingStateScope,
    commands: &MergedCommitPlan,
    identity: &crate::mvcc::RelationalMutationProposalIdentity,
    symbols: PendingSymbolMeanings,
    content: &crate::content::RelationalContentAttachments,
) -> Result<AppliedMutationCore, MutationApplyDenial> {
    let plan = AuthoritativeApplyPlan {
        transaction_id: commands.transaction_id,
        version_id: identity.proposed_version_id(),
        merged_intents: commands.merged_intents.clone(),
    };
    let config = crate::config::data::MutationConfig {
        cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
        adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
        cross_context_policy: runtime.config.storage.cross_context_policy,
    };
    let mut applied = apply(
        execution,
        runtime,
        selected,
        &prepared.working_state,
        commands,
        &plan,
        &config,
        &prepared.schema_authority,
        identity,
        symbols,
        None,
    )?;
    let source = identity
        .source_interpretation()
        .expect("owner-issued prefix identity");
    applied.state.content = super::content::prepare(
        execution,
        runtime,
        selected.selected().state().root(),
        &prepared.schema_authority,
        content,
        &applied.state,
        &applied.outcome.effect.publication.changed_records,
        identity.proposed_version_id(),
        source.restoration_content(),
    )?;
    Ok(applied)
}
