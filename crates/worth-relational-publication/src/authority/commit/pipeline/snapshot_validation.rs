use super::history_binding::HistoryBoundCommitExecution;
use super::phase_backing::CommitPhaseBacking;
use worth_relational_contracts::internal_port as contracts;

pub(super) struct SnapshotValidatedCommitExecution {
    admitted: super::execution_admission::AdmittedCommitState,
    selected_branch_state: worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
    public_structural_summary: contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
    working_state: worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
    content: worth_relational_branch::internal_port::content::publication::ContentPublication,
    invariant_executions: Vec<worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult>,
    version_id: contracts::identity::data::primitives::VersionId,
    effect: worth_relational_mutation::internal_port::authority::mutation::effect::MutationEffect,
    created_entities: contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
    created_relations: contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
    mutation_identities:
        worth_relational_mutation::internal_port::authority::mutation::pending_identities::PendingMutationIdentities,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    schema_continuity: crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,
    source_interpretation:
        std::sync::Arc<worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>,
}

impl SnapshotValidatedCommitExecution {
    pub(super) fn has_codec_publication(&self) -> bool {
        self.admitted
            .validation_input()
            .recovered_codec_publication_custody()
            .is_some()
    }
    #[allow(clippy::type_complexity)]
    pub(super) fn into_parts(
        self,
    ) -> (
        super::execution_admission::AdmittedCommitState,
        worth_relational_branch::internal_port::branch::root_selection::SelectedRelationalBranchState,
        contracts::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
        worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState,
        worth_relational_branch::internal_port::content::publication::ContentPublication,
        Vec<worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult>,
        contracts::identity::data::primitives::VersionId,
        worth_relational_mutation::internal_port::authority::mutation::effect::MutationEffect,
        contracts::transactions::data::outcomes::created_entity_bindings::CommitCreatedEntityBindings,
        contracts::transactions::data::outcomes::created_relation_bindings::CommitCreatedRelationBindings,
        worth_relational_mutation::internal_port::authority::mutation::pending_identities::PendingMutationIdentities,
        crate::authority::commit::phases::history::ResolvedCommitHistory,
        crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,

        std::sync::Arc<worth_relational_mutation::internal_port::validation::candidate_source::interpretation::NativeManagedSourceInterpretation>,
    ){
        (
            self.admitted,
            self.selected_branch_state,
            self.public_structural_summary,
            self.working_state,
            self.content,
            self.invariant_executions,
            self.version_id,
            self.effect,
            self.created_entities,
            self.created_relations,
            self.mutation_identities,
            self.history,
            self.schema_continuity,
            self.source_interpretation,
        )
    }
}

pub(super) fn validate_snapshot_publication(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    history_bound: CommitPhaseBacking<HistoryBoundCommitExecution>,
) -> Result<
    CommitPhaseBacking<SnapshotValidatedCommitExecution>,
    contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError,
> {
    // Producer temporaries retire before funding the retained successor,
    // matching the former caller-side allocation boundary.
    let result = {
        let mut history_bound = history_bound.into_inner();
        super::consult_stage(execution, super::CommitStage::Snapshot)?;
        let (log, _) = history_bound
            .mutated_mut()
            .prepared
            .admitted
            .commit_phase_state();
        log.begin_phase(contracts::transactions::data::commit_log::CommitPhase::InvariantPostCheck);
        log.complete_phase(
            contracts::transactions::data::commit_log::CommitPhase::InvariantPostCheck,
        );
        into_snapshot_validated_execution(history_bound)
    };
    CommitPhaseBacking::new(execution, result)
}

fn into_snapshot_validated_execution(
    history_bound: HistoryBoundCommitExecution,
) -> SnapshotValidatedCommitExecution {
    let (mutated, history, continuity) = history_bound.into_parts();
    let version_id = mutated.version_id();
    let selected_branch_state = mutated.prepared.selected_branch_state().clone();
    let super::mutation_execution::MutatedCommitExecution {
        prepared,
        working_state,
        content,
        invariant_executions,
        outcome,
        mutation_identities,
        _aggregate: _,
        source_interpretation,
    } = mutated;
    let super::draft_execution::PreparedCommitState {
        admitted,
        public_structural_summary,
        ..
    } = prepared;
    let worth_relational_mutation::internal_port::authority::mutation::execution::MutationApplyOutcome {
        effect,
        created_entities,
        created_relations,
        ..
    } = outcome;
    SnapshotValidatedCommitExecution {
        admitted,
        selected_branch_state,
        public_structural_summary,
        working_state,
        content,
        invariant_executions,
        version_id,
        effect,
        created_entities,
        created_relations,
        mutation_identities,
        history,
        schema_continuity: continuity,
        source_interpretation,
    }
}
