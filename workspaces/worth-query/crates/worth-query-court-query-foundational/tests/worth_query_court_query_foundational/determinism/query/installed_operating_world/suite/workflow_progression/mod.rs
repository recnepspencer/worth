pub(crate) use worth_query_testkit_query::facade::query::suite::workflow_progression::*;

#[test]
fn independent_stage_order_converges_to_one_semantic_trace() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let left_first = complete_trace(request_execution, "workflow-left-first", ["left", "right"]);
    let right_first = complete_trace(request_execution, "workflow-right-first", ["right", "left"]);
    assert_eq!(left_first, right_first);
}
