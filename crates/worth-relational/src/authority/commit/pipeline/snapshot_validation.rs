use super::history_binding::HistoryBoundCommitExecution;
use super::phase_backing::CommitPhaseBacking;

pub(super) struct SnapshotValidatedCommitExecution {
    admitted: super::execution_admission::AdmittedCommitState,
    selected_branch_state: crate::branch::SelectedRelationalBranchState,
    public_structural_summary: crate::transactions::data::CommitStructuralSummary,
    working_state: crate::storage::overlay::WorkingState,
    invariant_executions: Vec<crate::validation::engine::InvariantExecutionResult>,
    version_id: crate::identity::data::VersionId,
    effect: crate::authority::mutation::MutationEffect,
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::CommitCreatedRelationBindings,
    mutation_identities:
        crate::authority::commit::phases::applied_mutation::PendingMutationIdentities,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    merge_parent_branches: Vec<crate::history::data::BranchId>,
    additional_diagnostics_entries: Vec<crate::diagnostics::data::RelationalDiagnosticsEntry>,
    merge_execution_authority: Option<crate::transactions::data::PublishedMergeExecutionAuthority>,
    schema_continuity: crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,
    deferred_diagnostic_artifacts: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    source_interpretation:
        std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
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
        crate::branch::SelectedRelationalBranchState,
        crate::transactions::data::CommitStructuralSummary,
        crate::storage::overlay::WorkingState,
        Vec<crate::validation::engine::InvariantExecutionResult>,
        crate::identity::data::VersionId,
        crate::authority::mutation::MutationEffect,
        crate::transactions::data::CommitCreatedEntityBindings,
        crate::transactions::data::CommitCreatedRelationBindings,
        crate::authority::commit::phases::applied_mutation::PendingMutationIdentities,
        crate::authority::commit::phases::history::ResolvedCommitHistory,
        Vec<crate::history::data::BranchId>,
        Vec<crate::diagnostics::data::RelationalDiagnosticsEntry>,
        Option<crate::transactions::data::PublishedMergeExecutionAuthority>,
        crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
        std::sync::Arc<crate::validation::candidate_source::NativeManagedSourceInterpretation>,
    ) {
        (
            self.admitted,
            self.selected_branch_state,
            self.public_structural_summary,
            self.working_state,
            self.invariant_executions,
            self.version_id,
            self.effect,
            self.created_entities,
            self.created_relations,
            self.mutation_identities,
            self.history,
            self.merge_parent_branches,
            self.additional_diagnostics_entries,
            self.merge_execution_authority,
            self.schema_continuity,
            self.deferred_diagnostic_artifacts,
            self.source_interpretation,
        )
    }
}

pub(super) fn validate_snapshot_publication(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    _runtime: &crate::runtime::RelationalPreparationRuntime,
    history_bound: CommitPhaseBacking<HistoryBoundCommitExecution>,
    _control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<
    CommitPhaseBacking<SnapshotValidatedCommitExecution>,
    crate::transactions::data::TransactionCommitError,
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
        log.begin_phase(crate::transactions::data::CommitPhase::InvariantPostCheck);
        log.complete_phase(crate::transactions::data::CommitPhase::InvariantPostCheck);
        into_snapshot_validated_execution(history_bound)
    };
    CommitPhaseBacking::new(execution, result)
}

fn into_snapshot_validated_execution(
    history_bound: HistoryBoundCommitExecution,
) -> SnapshotValidatedCommitExecution {
    let (mutated, history, merge_parents, diagnostics, merge_authority, continuity, deferred) =
        history_bound.into_parts();
    let version_id = mutated.version_id();
    let selected_branch_state = mutated.prepared.selected_branch_state().clone();
    let super::mutation_execution::MutatedCommitExecution {
        prepared,
        working_state,
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
    let crate::authority::mutation::MutationApplyOutcome {
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
        invariant_executions,
        version_id,
        effect,
        created_entities,
        created_relations,
        mutation_identities,
        history,
        merge_parent_branches: merge_parents,
        additional_diagnostics_entries: diagnostics,
        merge_execution_authority: merge_authority,
        schema_continuity: continuity,
        deferred_diagnostic_artifacts: deferred,
        source_interpretation,
    }
}
