//! run phase sixteen matrix.
use super::*;

pub(super) fn run_phase_sixteen_matrix(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
    label: &'static str,
) -> ConcurrentMatrixRun {
    run_phase_sixteen_matrix_with_interleaving(
        request_execution,
        label,
        SubmitterInterleaving::Ascending,
    )
}
