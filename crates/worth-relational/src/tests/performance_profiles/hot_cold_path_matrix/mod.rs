use super::*;

mod chip_recovery_compile;
mod chip_rich_recovery_compile;
mod geometry_publication_replay_truth;
mod geometry_replay_reconstruction;

#[test]
#[ignore = "performance baseline capture; run with -- --ignored --nocapture --test-threads=1"]
fn perf_hot_cold_path_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let suite = "hot_cold_path_matrix";

    geometry_replay_reconstruction::certify_geometry_hot_commit_vs_replay_reconstruction(
        execution, suite,
    );
    chip_recovery_compile::certify_chip_hot_compile_vs_recovery_compile(execution, suite);
    geometry_publication_replay_truth::certify_geometry_rich_publication_hot_vs_replay_truth(
        execution, suite,
    );
    chip_rich_recovery_compile::certify_chip_rich_compile_hot_vs_recovery_compile(execution, suite);
}
