//! run phase sixteen serialized replay.
use super::*;

pub(super) fn run_phase_sixteen_serialized_replay(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &'static str,
) -> ConcurrentMatrixRun {
    let mut state = ConcurrentMatrixState::new(request_execution, label);
    state.capture_unpublished_posture(request_execution);
    state.record_branch_basis(request_execution, "phase16-branch-a");
    state.record_preview_discard(request_execution);
    state.execute_serialized_reader_matrix(request_execution);
    for submission in planned_phase_sixteen_submissions() {
        state.submit_planned(request_execution, submission);
    }
    state.reconsume_published_artifacts();
    state.record_preview_promotion(request_execution);
    state.record_branch_basis(request_execution, "phase16-branch-b");
    state.finish(request_execution)
}
