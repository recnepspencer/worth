use super::rejection::elapsed_micros;
use crate::authority::commit::phases::prepare::working_state_metrics::record_preparation_counters;
use crate::transactions::data::commit_log::CommitLog;
use crate::transactions::data::commit_log::CommitPhase;
use crate::transactions::data::outcomes::commit_result::CommitPhaseTiming;

pub(super) fn record_draft_preparation_phase(
    instrumentation: &crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    commit_log: &mut CommitLog,
    phase_timing: &mut CommitPhaseTiming,
    working_state: &crate::storage::overlay::WorkingState,
    structural_summary: &crate::authority::commit::structural_summary::CommitStructuralSummary,
    public_structural_summary: &crate::transactions::data::outcomes::commit_summaries::CommitStructuralSummary,
) {
    commit_log.begin_phase(CommitPhase::DraftPreparation);
    let phase_started = std::time::Instant::now();
    record_preparation_counters(instrumentation, working_state, structural_summary);
    commit_log.record_structural_summary(public_structural_summary);
    commit_log.complete_phase(CommitPhase::DraftPreparation);
    phase_timing.working_state_preparation_micros = elapsed_micros(phase_started);
}
