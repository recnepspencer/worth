//! One ordinary mutation application owner, shared by a complete ordinary plan
//! and the authored prefix of a coupled producer assembly.
use super::*;
use worth_relational_contracts::internal_port as contracts;

pub struct AppliedMutationCore {
    pub state: WorkingState,
    pub content: worth_relational_branch::internal_port::content::publication::ContentPublication,
    pub outcome: MutationApplyOutcome,
    pub identities: PendingMutationIdentities,
    pub aggregate: Option<
        worth_relational_mutation::internal_port::aggregate_identity::proposal::AggregateProposal,
    >,
    pub coupled_publication: Option<
        crate::mvcc::publication::application_publication::RequiredOutputPublicationAdmission,
    >,
}

impl AppliedMutationCore {
    pub fn state(&self) -> &WorkingState {
        &self.state
    }
    pub fn candidate_symbols(&self) -> &PendingSymbolMeanings {
        &self.identities.symbols
    }
}

pub(crate) fn apply<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<AppliedMutationInputs<'a>>,
    selected_state: &SelectedRelationalBranchRead<'_>,
    working_state: &WorkingState,
    merged_plan: &MergedCommitPlan,
    apply_plan: &AuthoritativeApplyPlan,
    config: &contracts::config::data::policies::MutationConfig,
    schema: &worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
    identity: &worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    mut symbols: PendingSymbolMeanings,
    materialization: Option<contracts::mvcc::transaction::intent::materialization_mode::RelationalMaterializationTransactionMode>,
) -> Result<AppliedMutationCore, MutationApplyDenial> {
    let inputs = inputs.into();
    let version_id = identity.proposed_version_id();
    worth_relational_mutation::internal_port::aggregate_identity::guard::ordinary_plan(
        execution,
        inputs.work.as_ref(),
        schema.registry(),
        &worth_relational_mutation::internal_port::validation::engine::state_view::InvariantStateView::new(
            selected_state,
            selected_state.version_id(),
        ),
        &merged_plan.merged_intents,
        materialization,
    )?;
    let mut state = working_state.clone();
    let allowance = branch_local_delete_allowance_for_plan(selected_state, &state, merged_plan);
    let mut records = inputs.record_identity.begin_allocations();
    let outcome = apply_plan_with_publication_edges(
        execution,
        &mut state,
        apply_plan,
        config,
        schema.registry(),
        schema.aspect_plans(),
        &mut symbols,
        allowance,
        &mut records,
        inputs.work.clone(),
        inputs
            .authored_coupling
            .map(|binding| &binding.owners.deferred_edges),
    )?;
    worth_relational_mutation::internal_port::authority::mutation::record_changes::adjacency::apply_adjacency_deltas_for_request(
        execution,
        &mut state,
        &outcome.effect.adjacency.deltas,
        version_id,
    )?;
    let aggregate = worth_relational_mutation::internal_port::aggregate_identity::proposal::AggregateProposal::capture(
        schema,
        selected_state.selected(),
        identity,
        &outcome.effect.publication.changed_records,
        &outcome.effect.adjacency.deltas,
        outcome.effect.publication.has_entity_deletions,
        materialization,
    );
    Ok(AppliedMutationCore {
        content: Default::default(),
        aggregate,
        state,
        outcome,
        identities: PendingMutationIdentities { records, symbols },
        coupled_publication: None,
    })
}

/// The authored prefix uses the same mutation owner and the enclosing round's
/// actual Native proposal identity, allocation stream and symbol reservations.
pub fn apply_authored_prefix<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<AppliedMutationInputs<'a>>,
    selected: &SelectedRelationalBranchRead<'_>,
    prepared: &super::super::prepare::PreparedWorkingStateScope,
    commands: &MergedCommitPlan,
    identity: &worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    symbols: PendingSymbolMeanings,
    content: &worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
) -> Result<AppliedMutationCore, MutationApplyDenial> {
    let inputs = inputs.into();
    // This apply owner needs its own copied intent storage. The authored plan
    // retains its original reservation independently until coupled completion.
    let _plan_memory = worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::reserve_copy(
        execution,
        worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::intent_bytes(&commands.merged_intents),
    )?;
    let plan = AuthoritativeApplyPlan {
        transaction_id: commands.transaction_id,
        version_id: identity.proposed_version_id(),
        merged_intents: commands.merged_intents.clone(),
    };
    let config = inputs.mutation_config();
    let mut applied = apply(
        execution,
        inputs,
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
    applied.content = super::content::prepare(
        execution,
        inputs,
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
