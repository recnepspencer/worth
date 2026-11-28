use super::*;

mod chip_recovery_compile;
mod chip_rich_recovery_compile;
mod geometry_publication_replay_truth;
mod geometry_replay_reconstruction;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_hot_cold_path_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    geometry_replay_reconstruction::certify_geometry_hot_commit_vs_replay_reconstruction(execution);
    chip_recovery_compile::certify_chip_hot_commit_and_recovery(execution);
    geometry_publication_replay_truth::certify_geometry_rich_publication_hot_vs_replay_truth(
        execution,
    );
    chip_rich_recovery_compile::certify_chip_rich_hot_commit_and_recovery(execution);
}
