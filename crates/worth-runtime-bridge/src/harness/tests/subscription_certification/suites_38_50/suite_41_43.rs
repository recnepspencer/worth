use super::super::support::*;
use crate::policy::BridgeRuntimePolicy;

#[test]
fn bridge_harness_subscription_suite_41_to_43_rows_are_present() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let artifact = sealed_phase_18_closeout(BridgeRuntimePolicy::development(), resource_request);
    let rows = artifact.support_matrix().rows();
    assert!(rows
        .iter()
        .any(|row| row.suite_id().as_str() == "suite_41_ordering_hostility"));
    assert!(rows
        .iter()
        .any(|row| row.suite_id().as_str() == "suite_42_stale_checkpoint"));
    assert!(rows
        .iter()
        .any(|row| row.suite_id().as_str() == "suite_43_bundle_insufficiency"));
}
