//! Graph selection uses the same Source, content, aggregate and identity owners.
use super::*;
use crate::mvcc::owned_graph_selection::PreparedOwnedGraphSelection;

pub(crate) fn apply(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalPreparationRuntime,
    selected_state: &SelectedRelationalBranchRead<'_>,
    working_state: &WorkingState,
    retained: &PreparedOwnedGraphSelection,
    schema: &crate::branch::RelationalBranchRootSchemaAuthority,
    identity: &crate::mvcc::RelationalMutationProposalIdentity,
    mut symbols: PendingSymbolMeanings,
    source_interpretation: std::sync::Arc<
        crate::validation::candidate_source::NativeManagedSourceInterpretation,
    >,
    content: &crate::content::RelationalContentAttachments,
) -> Result<AppliedMutationDraft, MutationApplyDenial> {
    require_original_source_interpretation(identity, &source_interpretation)?;
    let mut state = working_state.clone();
    let mut records = runtime.record_identity.begin_allocations();
    let config = crate::config::data::MutationConfig {
        cascade_delete_policy: runtime.config.storage.cascade_delete_policy,
        adjacency_policy: runtime.config.storage.adjacency_policy.clone(),
        cross_context_policy: runtime.config.storage.cross_context_policy,
    };
    let outcome = crate::authority::mutation::owned_graph_selection::apply(
        execution,
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
    crate::authority::mutation::apply_adjacency_deltas_for_request(
        execution,
        &mut state,
        &outcome.effect.adjacency.deltas,
        identity.proposed_version_id(),
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
        Some(
            retained
                .graph
                .entities()
                .selected_basis()
                .inner
                .root
                .retained_content(),
        ),
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
        outcome.effect.clone(),
        None,
    );
    Ok(AppliedMutationDraft {
        source_interpretation,
        aggregate,
        state,
        outcome,
        identities: PendingMutationIdentities { records, symbols },
    })
}
