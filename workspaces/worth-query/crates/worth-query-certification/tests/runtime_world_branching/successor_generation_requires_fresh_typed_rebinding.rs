//! successor generation requires fresh typed rebinding.
use super::*;

#[test]
fn successor_generation_requires_fresh_typed_rebinding() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut world = CourtroomWorld::publish("ready");
    let successor = std::sync::Arc::new(world.installation.successor_generation());

    let denial = world
        .application
        .reinstall_conditional_runtime_for_installation(
            request_execution,
            successor,
            world.application.current_world(),
        )
        .unwrap_err();

    assert_eq!(
        denial.kind(),
        primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind::RebindRequired
    );
}
