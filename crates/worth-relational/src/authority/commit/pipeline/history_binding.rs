use super::history_resolution_phase::resolve_authoritative_history_phase;
use super::mutation_execution::MutatedCommitExecution;
use crate::authority::commit::phases::schema_continuity::{
    prepare_schema_continuity_diagnostics, resolve_schema_continuity,
};
use crate::transactions::data::{CommitPhase, PublishedMergeExecutionAuthority};

pub(super) struct HistoryBoundCommitExecution {
    mutated: MutatedCommitExecution,
    history: crate::authority::commit::phases::history::ResolvedCommitHistory,
    merge_parent_branches: Vec<crate::history::data::BranchId>,
    additional_diagnostics_entries: Vec<crate::diagnostics::data::RelationalDiagnosticsEntry>,
    merge_execution_authority: Option<PublishedMergeExecutionAuthority>,
    schema_continuity: crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,
    deferred_diagnostic_artifacts: Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
}

impl HistoryBoundCommitExecution {
    pub(super) fn mutated_mut(&mut self) -> &mut MutatedCommitExecution {
        &mut self.mutated
    }

    pub(super) fn into_parts(
        self,
    ) -> (
        MutatedCommitExecution,
        crate::authority::commit::phases::history::ResolvedCommitHistory,
        Vec<crate::history::data::BranchId>,
        Vec<crate::diagnostics::data::RelationalDiagnosticsEntry>,
        Option<PublishedMergeExecutionAuthority>,
        crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan,
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
    ) {
        (
            self.mutated,
            self.history,
            self.merge_parent_branches,
            self.additional_diagnostics_entries,
            self.merge_execution_authority,
            self.schema_continuity,
            self.deferred_diagnostic_artifacts,
        )
    }
}

/// Required request borrow proves this work belongs to the caller, without retaining it.
pub(super) fn bind_commit_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    mut mutated: MutatedCommitExecution,
) -> Result<HistoryBoundCommitExecution, crate::transactions::data::TransactionCommitError> {
    super::consult_stage(execution, super::CommitStage::History)?;
    let version_id = mutated.version_id();
    let admitted = mutated.validated_mut().prepared_mut().admitted_mut();
    let merge_diagnostics_plan = admitted.merge_execution_diagnostics_plan().cloned();
    let (transaction_id, validation_input, _, merge_plan, commit_log, phase_timing) =
        admitted.phase_view().into_parts();
    let history = resolve_authoritative_history_phase(
        runtime,
        commit_log,
        phase_timing,
        transaction_id,
        validation_input,
        version_id,
        merge_plan,
    )?;
    let additional_diagnostics_entries = merge_plan
        .map(|plan| {
            vec![crate::merge::merge_execution_summary_entry(
                &plan.merge_execution_summary,
                &plan.structural_summary,
                history.commit_id,
            )]
        })
        .unwrap_or_default();
    let mut deferred_diagnostic_artifacts = if let (Some(plan), Some(diagnostics_plan)) =
        (merge_plan, merge_diagnostics_plan.as_ref())
    {
        vec![crate::merge::merge_execution_success_artifact(
            &plan.merge_execution_summary,
            diagnostics_plan,
            history.commit_id,
            runtime.config.diagnostics.profile.max_entries_per_artifact,
        )]
    } else {
        Vec::new()
    };
    let merge_execution_authority =
        merge_plan.map(PublishedMergeExecutionAuthority::from_merge_plan);
    let schema_continuity =
        resolve_schema_continuity(runtime, &history.branch_id, validation_input).map_err(
            |error| {
                super::rejection::attach_rejection(commit_log, CommitPhase::ArtifactAssembly, error)
            },
        )?;
    deferred_diagnostic_artifacts.extend(prepare_schema_continuity_diagnostics(
        &history.branch_id,
        &schema_continuity,
    ));
    let merge_parent_branches = validation_input.merge_parent_branch_ids();
    mutated.bind_aggregate_consequences(execution, runtime, history.commit_id)?;
    Ok(HistoryBoundCommitExecution {
        mutated,
        history,
        merge_parent_branches,
        additional_diagnostics_entries,
        merge_execution_authority,
        schema_continuity,
        deferred_diagnostic_artifacts,
    })
}
