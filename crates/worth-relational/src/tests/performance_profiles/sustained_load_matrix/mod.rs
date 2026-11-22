use super::*;

mod chip_global_step_endurance;
mod commit_query_churn;
mod mixed_topology_query_churn;
mod replay_window_drift;
mod retention_pass_drift;
mod rocketship_hot_update_endurance;
mod rocketship_propagation_endurance;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_sustained_load_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    commit_query_churn::certify_commit_query_churn_stability(execution);
    replay_window_drift::certify_replay_window_drift_stability(execution);
    retention_pass_drift::certify_retention_pass_drift_stability(execution);
    mixed_topology_query_churn::certify_mixed_topology_query_churn_stability(execution);
    rocketship_hot_update_endurance::certify_rocketship_hot_update_endurance(execution);
    rocketship_propagation_endurance::certify_rocketship_propagation_endurance(execution);
    chip_global_step_endurance::certify_chip_global_step_endurance(execution);
}
