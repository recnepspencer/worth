//! reconstruction panic restores runtime owners for retry.
use super::*;

#[test]
fn reconstruction_panic_restores_runtime_owners_for_retry() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = CourtroomWorld::publish("ready");
    world.reconstruction_panic.set(true);

    assert!(world.reinstall_conditional_runtime().is_err());

    world.reconstruction_panic.set(false);
    assert!(world.reinstall_conditional_runtime().is_ok());
    let receipt = observe(request_execution, &world);
    assert_eq!(receipt.committed_operation_count(), 1);
}
