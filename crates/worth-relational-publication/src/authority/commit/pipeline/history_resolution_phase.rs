use super::rejection::{attach_rejection, elapsed_micros};
use crate::authority::commit::phases::history::{resolve_commit_history, ResolvedCommitHistory};
use crate::transactions::data::outcomes::commit_result::CommitPhaseTiming;
use contracts::identity::data::primitives::VersionId;
use contracts::transactions::data::commit_log::CommitLog;
use contracts::transactions::data::commit_log::CommitPhase;
use contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_contracts::internal_port as contracts;

pub(super) fn resolve_authoritative_history_phase(
    commit_log: &mut CommitLog,
    phase_timing: &mut CommitPhaseTiming,
    basis: &AdmittedRelationalBranchBasis,
    version_id: VersionId,
    commit_id: contracts::history::data::coordinates::CommitId,
) -> Result<ResolvedCommitHistory, TransactionCommitError> {
    commit_log.begin_phase(CommitPhase::HistoryResolution);
    let phase_started = worth_execution::MonotonicClock::system().now();
    let history = resolve_commit_history(basis, version_id, commit_id)
        .map_err(|error| attach_rejection(commit_log, CommitPhase::HistoryResolution, error))?;
    let history_summary = history.summary();
    commit_log.record_history_resolution(&history_summary);
    commit_log.complete_phase(CommitPhase::HistoryResolution);
    phase_timing.history_resolution_micros = elapsed_micros(phase_started);
    Ok(history)
}
