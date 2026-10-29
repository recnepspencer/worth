pub(crate) use worth_query_testkit_query::facade::query::suite::operation_reexecution::*;

#[test]
fn replay_comparator_must_be_installed_before_any_replay_can_execute() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let denial =
        match missing_replay_comparator_workspace(request_execution, "missing-replay-comparator") {
            Ok(_) => panic!("replay contract without a comparator object must not install"),
            Err(denial) => denial,
        };
    assert!(denial
        .message()
        .contains("workflow replay comparator disagrees with installed semantics"));
}

pub(crate) mod stage_retry;
