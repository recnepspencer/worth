//! run phase sixteen matrix with interleaving.
use super::*;

pub(super) fn run_phase_sixteen_matrix_with_interleaving(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &'static str,
    interleaving: SubmitterInterleaving,
) -> ConcurrentMatrixRun {
    let mut state = ConcurrentMatrixState::new(request_execution, label);
    state.capture_unpublished_posture(request_execution);
    state.record_branch_basis(request_execution, "phase16-branch-a");
    state.record_preview_discard(request_execution);
    state.execute_concurrent_reader_and_submitter_matrix(request_execution, interleaving);
    state.reconsume_published_artifacts();
    state.record_preview_promotion(request_execution);
    state.record_branch_basis(request_execution, "phase16-branch-b");
    state.finish(request_execution)
}
