use super::*;
mod branch_divergence;
mod feature_adoption_execution;
mod persisted_commit_floor;
mod phase_timing;
mod planning_divergent_update;
mod prepare_execute_split;
mod verify_execute_split;
mod zero_diagnostics_execution;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_merge_lineage_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    planning_divergent_update::certify_merge_planning_divergent_update(execution);
    feature_adoption_execution::certify_merge_execution_feature_adoption(execution);
    zero_diagnostics_execution::certify_merge_execution_zero_diagnostics_budget(execution);
    prepare_execute_split::certify_merge_prepare_vs_execute_feature_adoption(execution);
    persisted_commit_floor::certify_merge_execution_vs_persisted_commit_floor(execution);
    verify_execute_split::certify_merge_verify_vs_execute_feature_adoption(execution);
    phase_timing::certify_merge_execute_phase_timing_feature_adoption(execution);
    branch_divergence::certify_lineage_branch_divergence_breadth(execution);
}
