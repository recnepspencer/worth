//! This test crate is the host of these standalone Bridge requests.
pub(crate) fn serial_request() -> worth_execution::SerialRequest {
    let policy =
        worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational().request_policy();
    worth_execution::SerialRequest::from_policy(
        &policy,
        worth_execution::CancellationToken::new(),
        None,
    )
}
