use super::history_binding::HistoryBoundCommitExecution;
use super::invariant_phase::enforce_snapshot_publication_phase;

pub(super) struct SnapshotValidatedCommitExecution {
    admitted: super::execution_admission::AdmittedCommitExecution,
    selected_branch_state: crate::branch::SelectedRelationalBranchState,
    public_structural_summary: crate::transactions::data::CommitStructuralSummary,
    working_state: crate::storage::overlay::WorkingState,
    invariant_executions: Vec<crate::validation::engine::InvariantExecutionResult>,
    version_id: crate::identity::data::VersionId,
    effect: crate::authority::mutation::MutationEffect,
    created_entities: crate::transactions::data::CommitCreatedEntityBindings,
    created_relations: crate::transactions::data::CommitCreatedRelationBindings,
    record_allocations: crate::runtime::PendingRecordAllocations,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    merge_parent_branches: Vec<crate::history::data::BranchId>,
    additional_diagnostics_entries: Vec<crate::diagnostics::data::RelationalDiagnosticsEntry>,
    merge_execution_authority: Option<crate::transactions::data::PublishedMergeExecutionAuthority>,
    schema_continuity: crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,
    deferred_diagnostic_artifacts: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
}

impl SnapshotValidatedCommitExecution {
    #[allow(clippy::type_complexity)]
    pub(super) fn into_parts(
        self,
    ) -> (
        super::execution_admission::AdmittedCommitExecution,
        crate::branch::SelectedRelationalBranchState,
        crate::transactions::data::CommitStructuralSummary,
        crate::storage::overlay::WorkingState,
        Vec<crate::validation::engine::InvariantExecutionResult>,
        crate::identity::data::VersionId,
        crate::authority::mutation::MutationEffect,
        crate::transactions::data::CommitCreatedEntityBindings,
        crate::transactions::data::CommitCreatedRelationBindings,
        crate::runtime::PendingRecordAllocations,
        crate::authority::commit::phases::history::ResolvedCommitHistory,
        Vec<crate::history::data::BranchId>,
        Vec<crate::diagnostics::data::RelationalDiagnosticsEntry>,
        Option<crate::transactions::data::PublishedMergeExecutionAuthority>,
        crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
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
            self.record_allocations,
            self.history,
            self.merge_parent_branches,
            self.additional_diagnostics_entries,
            self.merge_execution_authority,
            self.schema_continuity,
            self.deferred_diagnostic_artifacts,
        )
    }
}

pub(super) fn validate_snapshot_publication(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut history_bound: HistoryBoundCommitExecution,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<SnapshotValidatedCommitExecution, crate::transactions::data::TransactionCommitError> {
    enforce_snapshot_invariant(runtime, &mut history_bound, lease, control)?;
    Ok(into_snapshot_validated_execution(history_bound))
}

fn enforce_snapshot_invariant(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    history_bound: &mut HistoryBoundCommitExecution,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<(), crate::transactions::data::TransactionCommitError> {
    let mutated = history_bound.mutated_mut();
    let version_id = mutated.version_id();
    let selected_branch_state = mutated
        .validated_mut()
        .prepared_mut()
        .selected_branch_state()
        .clone();
    let proposal_identity = mutated
        .validated_mut()
        .prepared_mut()
        .proposal_identity()
        .clone();
    let (admitted, working_state) = mutated.validated_mut().prepared_mut().mutation_parts();
    let prevalidated_snapshot_publication = admitted.take_prevalidated_snapshot_publication();
    let (_, _, merged_plan, _, commit_log, phase_timing) = admitted.phase_view().into_parts();
    let invariant = enforce_snapshot_publication_phase(
        runtime,
        commit_log,
        phase_timing,
        &selected_branch_state,
        working_state,
        version_id,
        merged_plan,
        Some(&proposal_identity),
        prevalidated_snapshot_publication,
        lease,
        control,
    )?;
    mutated.validated_mut().push_invariant(invariant);
    Ok(())
}

fn into_snapshot_validated_execution(
    history_bound: HistoryBoundCommitExecution,
) -> SnapshotValidatedCommitExecution {
    let (mutated, history, merge_parents, diagnostics, merge_authority, continuity, deferred) =
        history_bound.into_parts();
    let (validated, version_id, effect, created_entities, created_relations, record_allocations) =
        mutated.into_parts();
    let (prepared, invariant_executions) = validated.into_parts();
    let selected_branch_state = prepared.selected_branch_state().clone();
    let (admitted, public_structural_summary, working_state) = prepared.into_parts();
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
        record_allocations,
        history,
        merge_parent_branches: merge_parents,
        additional_diagnostics_entries: diagnostics,
        merge_execution_authority: merge_authority,
        schema_continuity: continuity,
        deferred_diagnostic_artifacts: deferred,
    }
}
