//! Graph selection uses the same Source, content, aggregate and identity owners.
use super::*;
use worth_relational_mutation::internal_port::mvcc::owned_graph_selection::prepared::PreparedOwnedGraphSelection;

pub fn apply<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<AppliedMutationInputs<'a>>,
    selected_state: &SelectedRelationalBranchRead<'_>,
    working_state: &WorkingState,
    retained: &PreparedOwnedGraphSelection,
    schema: &worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
    identity: &worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    mut symbols: PendingSymbolMeanings,
    source_interpretation: std::sync::Arc<
        worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation,
    >,
    content: &worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
) -> Result<AppliedMutationDraft, MutationApplyDenial> {
    let inputs = inputs.into();
    require_original_source_interpretation(identity, &source_interpretation)?;
    let mut state = working_state.clone();
    let mut records = inputs.record_identity.begin_allocations();
    let config = inputs.mutation_config();
    let outcome = worth_relational_mutation::internal_port::authority::mutation::owned_graph_selection::apply(
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
    )?;
    worth_relational_mutation::internal_port::authority::mutation::record_changes::adjacency::apply_adjacency_deltas_for_request(
        execution,
        &mut state,
        &outcome.effect.adjacency.deltas,
        identity.proposed_version_id(),
    )?;
    let prepared_content = content::prepare(
        execution,
        inputs,
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
                .owner_inner()
                .root
                .retained_content(),
        ),
    )?;
    source_interpretation
        .bind_content(
            execution,
            prepared_content.directory().map_err(|_| {
                worth_relational_branch::internal_port::content::publication::commit_denial(
                    worth_relational_contracts::internal_port::content::denial::RelationalContentDenial::EncodingInconsistency,
                )
            })?,
            schema,
        )
        .map_err(worth_relational_branch::internal_port::content::publication::commit_denial)?;
    let aggregate = worth_relational_mutation::internal_port::aggregate_identity::proposal::AggregateProposal::capture(
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
        aggregate,
        state,
        outcome,
        identities: PendingMutationIdentities { records, symbols },
    })
}
