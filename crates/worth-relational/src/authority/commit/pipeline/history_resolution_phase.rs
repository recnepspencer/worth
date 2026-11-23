use super::rejection::{attach_rejection, elapsed_micros};
use crate::authority::commit::phases::history::{
    resolve_commit_history, resolve_commit_history_for_merge, CommitHistoryPreparation,
    ResolvedCommitHistory,
};
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::identity::data::VersionId;
use crate::transactions::data::commit_log::CommitLog;
use crate::transactions::data::commit_log::CommitPhase;
use crate::transactions::data::outcomes::commit_result::CommitPhaseTiming;
use crate::transactions::data::outcomes::plan_artifacts::MergeCommitMutationPlan;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;

pub(super) fn resolve_authoritative_history_phase(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    preparation: CommitHistoryPreparation<'_>,
    commit_log: &mut CommitLog,
    phase_timing: &mut CommitPhaseTiming,
    basis: &AdmittedRelationalBranchBasis,
    merge_parents: &[AdmittedRelationalBranchBasis],
    version_id: VersionId,
    commit_id: crate::history::data::CommitId,
    merge_history_plan: Option<&MergeCommitMutationPlan>,
) -> Result<ResolvedCommitHistory, TransactionCommitError> {
    commit_log.begin_phase(CommitPhase::HistoryResolution);
    let phase_started = std::time::Instant::now();
    let history = match merge_history_plan {
        None => resolve_commit_history(
            execution,
            &preparation,
            basis,
            merge_parents,
            version_id,
            commit_id,
        ),
        Some(plan) => resolve_commit_history_for_merge(basis, plan, version_id, commit_id),
    }
    .map_err(|error| attach_rejection(commit_log, CommitPhase::HistoryResolution, error))?;
    let history_summary = history.summary();
    commit_log.record_history_resolution(&history_summary);
    commit_log.complete_phase(CommitPhase::HistoryResolution);
    phase_timing.history_resolution_micros = elapsed_micros(phase_started);
    Ok(history)
}
