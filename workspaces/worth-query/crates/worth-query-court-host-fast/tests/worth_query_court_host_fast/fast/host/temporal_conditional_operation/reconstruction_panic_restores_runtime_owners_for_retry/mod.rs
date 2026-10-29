pub(crate) use worth_query_worlds::facade::host::temporal_conditional_operation::reconstruction_panic_restores_runtime_owners_for_retry::*;

#[test]
fn reconstruction_panic_restores_runtime_owners_for_retry() {
    let host_request = worth_query_worlds::facade::host::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = CourtroomWorld::publish("ready");
    world.reconstruction_panic.set(true);

    assert!(world.reinstall_conditional_runtime().is_err());

    world.reconstruction_panic.set(false);
    assert!(world.reinstall_conditional_runtime().is_ok());
    let receipt = observe(request_execution, &world);
    assert_eq!(receipt.committed_operation_count(), 1);
}
