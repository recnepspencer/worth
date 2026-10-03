use super::super::rejection::{attach_rejection, elapsed_micros};
use super::preparation::{
    prepare_publication_artifacts, PublicationPreparation, PublicationPreparationInput,
};
use crate::authority::commit::phases::publication::enforce_patch_budget;
use crate::authority::commit::phases::schema_continuity::SchemaContinuityPlan;
use crate::authority::commit::publication::assemble_patch;
use crate::authority::mutation::MutationEffect;
use crate::commit_strategies::data::StrategyCommitArtifactBundle;
use crate::diagnostics::data::RelationalDiagnosticsEntry;
use crate::history::data::{BranchId, CommitId, RelationalCommitReceipt};
use crate::identity::data::VersionId;
use crate::runtime::RelationalPreparationRuntime;
use crate::transactions::data::{
    CommitLog, CommitPatchBudgetSummary, CommitPhase, CommitPhaseTiming, MergedCommitPlan,
    PublishedMergeExecutionAuthority, TransactionCommitError,
};

pub(super) struct ArtifactAssemblyInput<'a> {
    pub(super) working_state: &'a mut crate::storage::overlay::WorkingState,
    pub(super) selected_branch_state: &'a crate::branch::SelectedRelationalBranchState,
    pub(super) effect: MutationEffect,
    pub(super) commit_reference: &'a RelationalCommitReceipt,
    pub(super) branch_id: &'a BranchId,
    pub(super) version_id: VersionId,
    pub(super) merge_parent_branches: &'a [BranchId],
    pub(super) merge_base_commits: &'a [CommitId],
    pub(super) merged_plan: &'a MergedCommitPlan,
    pub(super) record_allocations: &'a [crate::history::data::CanonicalRecordAllocation],
    pub(super) strategy_commit_artifacts: Option<StrategyCommitArtifactBundle>,
    pub(super) merge_execution_authority: Option<PublishedMergeExecutionAuthority>,
    pub(super) schema_continuity: &'a SchemaContinuityPlan,
    pub(super) additional_diagnostics_entries: Vec<RelationalDiagnosticsEntry>,
    pub(super) deferred_diagnostic_artifacts:
        Vec<crate::diagnostics::data::RelationalDiagnosticArtifact>,
}

pub(super) fn assemble_authoritative_publication_phase(
    runtime: &RelationalPreparationRuntime,
    commit_log: &mut CommitLog,
    phase_timing: &mut CommitPhaseTiming,
    input: ArtifactAssemblyInput<'_>,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
) -> Result<PublicationPreparation, TransactionCommitError> {
    commit_log.begin_phase(CommitPhase::ArtifactAssembly);
    let phase_started = std::time::Instant::now();
    let mut effect = input.effect;
    let patch_fragments = std::mem::take(&mut effect.publication.patch_fragments);
    let patch = assemble_patch(runtime, patch_fragments, lease)?;
    let patch_budget_summary = CommitPatchBudgetSummary {
        patch_record_count: patch.authoritative_record_patches.len(),
        max_patch_records_per_commit: runtime
            .config
            .publication
            .policy
            .max_patch_records_per_commit,
    };
    commit_log.record_patch_budget(&patch_budget_summary);
    enforce_patch_budget(runtime, &patch)
        .map_err(|error| attach_rejection(commit_log, CommitPhase::ArtifactAssembly, error))?;
    let publication = prepare_publication_artifacts(
        runtime,
        PublicationPreparationInput {
            working_state: input.working_state,
            selected_branch_state: input.selected_branch_state,
            patch,
            commit_reference: input.commit_reference,
            branch_id: input.branch_id,
            version_id: input.version_id,
            merge_parent_branches: input.merge_parent_branches,
            merge_base_commits: input.merge_base_commits,
            merged_plan: input.merged_plan,
            record_allocations: input.record_allocations,
            strategy_artifacts: input.strategy_commit_artifacts,
            merge_execution_authority: input.merge_execution_authority,
            schema_continuity: input.schema_continuity,
            effect,
            additional_diagnostics_entries: input.additional_diagnostics_entries,
            deferred_diagnostic_artifacts: input.deferred_diagnostic_artifacts,
        },
        lease,
    )
    .map_err(|error| attach_rejection(commit_log, CommitPhase::ArtifactAssembly, error))?;
    record_publication_phase_artifacts(commit_log, &publication);
    commit_log.complete_phase(CommitPhase::ArtifactAssembly);
    phase_timing.artifact_assembly_micros = elapsed_micros(phase_started);
    Ok(publication)
}

fn record_publication_phase_artifacts(
    commit_log: &mut CommitLog,
    publication: &PublicationPreparation,
) {
    let (change_summary, aspect_summary, publication_summary) = publication.summaries();
    commit_log.record_changed_records(change_summary);
    commit_log.record_aspect_summary(aspect_summary);
    commit_log.record_publication_artifacts(publication_summary);
}
