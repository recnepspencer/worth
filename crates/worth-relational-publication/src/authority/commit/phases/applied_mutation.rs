//! The exact applied draft judged by invariants and later offered for publication.
use worth_relational_contracts::internal_port as contracts;
use crate::authority::commit::phases::mutation::branch_local_delete_allowance_for_plan;
use worth_relational_mutation::internal_port::authority::mutation::execution::{MutationApplyOutcome, apply_plan_with_publication_edges};
use worth_relational_mutation::internal_port::authority::mutation::round_denial::MutationApplyDenial;
use worth_relational_branch::internal_port::branch::selected_read::SelectedRelationalBranchRead;
use worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::reservations::PendingSymbolMeanings;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
use contracts::transactions::data::outcomes::commit_conflict::CommitConflictOwnerPort as _;
use worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentityOwnerPort as _;

use contracts::transactions::data::outcomes::plan_artifacts::AuthoritativeApplyPlan;
use contracts::transactions::data::outcomes::plan_artifacts::MergedCommitPlan;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

pub(crate) use worth_relational_mutation::internal_port::authority::mutation::pending_identities::PendingMutationIdentities;

pub mod aggregate_consequences;
pub mod inputs;
use inputs::AppliedMutationInputs;
pub mod aggregate_inputs;
pub mod aggregate_metrics;
pub mod coupled_return;
pub(crate) use coupled_return::AppliedCoreCustody;
pub use coupled_return::AppliedDraftCustody;
pub use coupled_return::CoupledPreparationCleanup;
pub mod content;
pub mod ordinary_apply;
pub use ordinary_apply::AppliedMutationCore;

pub struct AppliedMutationDraft {
    pub state: WorkingState,
    pub(crate) content: worth_relational_branch::internal_port::content::publication::ContentPublication,
    pub outcome: MutationApplyOutcome,
    pub(crate) identities: PendingMutationIdentities,
    pub(crate) aggregate: Option<worth_relational_mutation::internal_port::aggregate_identity::proposal::AggregateProposal>,
    pub source_interpretation:
        std::sync::Arc<worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>,
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
    pub fn state(&self) -> &WorkingState {
        &self.state
    }

    pub fn aggregate_source<'a>(
        &'a self,
        selected: SelectedRelationalBranchRead<'a>,
    ) -> Option<worth_relational_mutation::internal_port::aggregate_identity::proposal::AggregateProposalBorrow<'a>>{
        self.aggregate
            .as_ref()
            .map(|owner| owner.borrow(&self.state, selected))
    }

    pub fn prepare_aggregate<'a>(
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

    pub fn candidate_symbols(&self) -> &PendingSymbolMeanings {
        &self.identities.symbols
    }

    /// Consumes detached preflight record reservations (so reclaimed slots stay
    /// claimed across sequential preflight steps) and extracts the updated
    /// detached symbol binding and staged partition commits.
    pub fn into_detached_preflight_state(
        self,
    ) -> (
        worth_relational_storage::internal_port::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        std::collections::BTreeMap<
            contracts::identity::data::primitives::PartitionId,
            (
                worth_relational_storage::internal_port::storage::overlay::PartitionState,
                worth_relational_storage::internal_port::storage::overlay::PartitionMutationJournal,
            ),
        >,
    ){
        let symbols = self.identities.symbols.binding();
        self.identities.records.commit();
        let partitions = self.state.into_partition_commits().1;
        (symbols, partitions)
    }
}

pub fn apply_mutation_draft<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<AppliedMutationInputs<'a>>,
    selected_state: &SelectedRelationalBranchRead<'_>,
    working_state: &WorkingState,
    merged_plan: &MergedCommitPlan,
    schema: &worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority,
    identity: &worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    mut symbols: PendingSymbolMeanings,
    source_interpretation: std::sync::Arc<
        worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation,
    >,
    content: &worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
) -> Result<AppliedDraftCustody, MutationApplyDenial> {
    let inputs = inputs.into();
    require_original_source_interpretation(identity, &source_interpretation)?;
    let materialization = source_interpretation.materialization_mode();
    let version_id = identity.proposed_version_id();
    let _apply_memory = worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::reserve_copy(
        execution,
        worth_relational_mutation::internal_port::mvcc::transaction::normalized_plan::intent_bytes(&merged_plan.merged_intents),
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
                worth_relational_branch::internal_port::content::publication::commit_denial(
                    contracts::content::denial::RelationalContentDenial::EncodingInconsistency,
                )
            })?,
            schema,
        )
        .map_err(worth_relational_branch::internal_port::content::publication::commit_denial)?;
    Ok(core.into_draft(source_interpretation))
}

pub fn apply_retained_entity_selection_draft<'a>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    inputs: impl Into<AppliedMutationInputs<'a>>,
    selected_state: &SelectedRelationalBranchRead<'_>,
    working_state: &WorkingState,
    retained: &worth_relational_mutation::internal_port::authority::commit::semantic_plan::retained_entity_payload::PreparedRetainedEntityPayload,
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
    worth_relational_mutation::internal_port::aggregate_identity::guard::retained_entity_selection(
        execution,
        inputs.work.as_ref(),
        schema.registry(),
        retained.meaning.kind(),
    )?;
    let mut state = working_state.clone();
    let mut records = inputs.record_identity.begin_allocations();
    let config = inputs.mutation_config();
    let outcome = contracts::execution::request_work::transaction_stage(
        execution,
        inputs.work.as_ref(),
        || {
            worth_relational_mutation::internal_port::authority::mutation::retained_entity_selection::apply(
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
        },
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
        Some(retained._source.owner_inner().root.retained_content()),
    )?;
    source_interpretation
        .bind_content(
            execution,
            prepared_content.directory().map_err(|_| {
                worth_relational_branch::internal_port::content::publication::commit_denial(
                    contracts::content::denial::RelationalContentDenial::EncodingInconsistency,
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
        state,
        outcome,
        aggregate,
        identities: PendingMutationIdentities { records, symbols },
    })
}

fn require_original_source_interpretation(
    identity: &worth_relational_mutation::internal_port::mvcc::validation::proposal_identity::RelationalMutationProposalIdentity,
    source: &std::sync::Arc<worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>,
) -> Result<(), MutationApplyDenial> {
    if identity
        .source_interpretation()
        .is_some_and(|owner| std::sync::Arc::ptr_eq(&owner, source))
    {
        return Ok(());
    }
    Err(contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError::conflict(
        contracts::transactions::data::outcomes::commit_conflict::CommitConflict::new(
            contracts::transactions::data::outcomes::conflict_class::ConflictClass::StaleValidationBasis {
                detail: "candidate Source belongs to another proposal owner".into(),
            },
        ),
    )
    .into())
}
