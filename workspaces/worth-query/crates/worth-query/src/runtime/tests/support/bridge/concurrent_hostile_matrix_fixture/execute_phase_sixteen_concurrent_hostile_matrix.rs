//! execute phase sixteen concurrent hostile matrix.
use super::*;

pub(in crate::runtime::tests) fn execute_phase_sixteen_concurrent_hostile_matrix(
    request_execution: &worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryConcurrentHostileMatrixArtifact {
    let primary = run_phase_sixteen_matrix(request_execution, "primary");
    let serialized_replay =
        run_phase_sixteen_serialized_replay(request_execution, "serialized-replay");
    let repeated = run_phase_sixteen_matrix_with_interleaving(
        request_execution,
        "repeated",
        SubmitterInterleaving::Descending,
    );
    let artifact_replay_equal = primary.raw_digest == serialized_replay.raw_digest;
    let repeated_run_equal = primary.raw_digest == repeated.raw_digest;
    WorthQueryConcurrentHostileMatrixArtifact::certify(
        primary.topology,
        primary.receipt_digests,
        primary.reader_result_digests,
        primary.published_artifact_digests,
        primary.preview_closeout_digests,
        primary.branch_basis_digests,
        primary.replay_digest,
        primary.counters,
        artifact_replay_equal,
        repeated_run_equal,
        true,
    )
}
