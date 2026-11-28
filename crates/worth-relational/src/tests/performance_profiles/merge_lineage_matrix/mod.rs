use super::*;
mod branch_divergence;

#[test]
#[ignore = "counted performance profile campaign; run with -- --ignored --nocapture --test-threads=1"]
fn perf_merge_lineage_matrix() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    branch_divergence::certify_lineage_branch_divergence_breadth(execution);
}
