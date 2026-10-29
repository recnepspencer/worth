use super::history_binding::HistoryBoundCommitExecution;
use super::invariant_phase::enforce_snapshot_publication_phase;

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
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut history_bound: HistoryBoundCommitExecution,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<SnapshotValidatedCommitExecution, crate::transactions::data::TransactionCommitError> {
    super::consult_stage(execution, super::CommitStage::Snapshot)?;
    enforce_snapshot_invariant(runtime, &mut history_bound, execution, control)?;
    Ok(into_snapshot_validated_execution(history_bound))
}

fn enforce_snapshot_invariant(
    runtime: &crate::runtime::RelationalPreparationRuntime,
    history_bound: &mut HistoryBoundCommitExecution,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    control: &crate::validation::engine::InvariantPreparationControl<'_, '_>,
) -> Result<(), crate::transactions::data::TransactionCommitError> {
    let mutated = history_bound.mutated_mut();
    let version_id = mutated.version_id();
    let selected_branch_state = mutated.prepared.selected_branch_state().clone();
    let proposal_identity = mutated.prepared.proposal_identity.clone();
    let admitted = &mut mutated.prepared.admitted;
    let basis = admitted.validation_input().basis().clone();
    let working_state = &mut mutated.working_state;
    let aggregate_plan = mutated
        .aggregate_admission
        .as_ref()
        .map(|admission| &admission.plan);
    let prevalidated_snapshot_publication = admitted.take_prevalidated_snapshot_publication();
    let (_, _, merged_plan, _, commit_log, phase_timing) = admitted.phase_view().into_parts();
    let merged_plan = aggregate_plan.unwrap_or(merged_plan);
    let (final_mutation, invariant, accepted_log, accepted_timing) =
        crate::storage::residency::probe_head(
            execution,
            &selected_branch_state.with_symbol_admission(&runtime.services.symbols),
            |partitions| {
                let selected = crate::branch::SelectedRelationalBranchRead::in_round(
                    &selected_branch_state,
                    partitions,
                );
                let aggregate_source = mutated
                    .aggregate
                    .as_ref()
                    .map(|owner| owner.borrow(working_state, selected));
                let mut round_log = commit_log.clone();
                let mut round_timing = phase_timing.clone();
                let final_mutation = if aggregate_plan.is_some() {
                    let answer = runtime
                        .invariant_authority()
                        .enforce_mutation_sensitive_for_working_state(
                            &selected,
                            working_state,
                            version_id,
                            merged_plan,
                            Some(&proposal_identity),
                            aggregate_source,
                            execution,
                            control,
                        )?;
                    round_log.record_invariant_outcomes(&answer);
                    Some(answer)
                } else {
                    None
                };
                let invariant = enforce_snapshot_publication_phase(
                    runtime,
                    &mut round_log,
                    &mut round_timing,
                    &selected,
                    working_state,
                    &mutated.outcome.created_entities,
                    version_id,
                    merged_plan,
                    Some(&proposal_identity),
                    aggregate_source,
                    prevalidated_snapshot_publication.clone(),
                    execution,
                    control,
                )?;
                Ok::<_, crate::transactions::data::TransactionCommitError>((
                    final_mutation,
                    invariant,
                    round_log,
                    round_timing,
                ))
            },
        )
        .map_err(|failure| {
            crate::authority::commit::preparation::storage_read_denial(&basis, failure)
        })??;
    *commit_log = accepted_log;
    *phase_timing = accepted_timing;
    if let Some(final_mutation) = final_mutation {
        mutated.invariant_executions.push(final_mutation);
    }
    mutated.invariant_executions.push(invariant);
    Ok(())
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
        aggregate: _,
        source_interpretation,
        aggregate_admission: _,
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
