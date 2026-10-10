use super::super::support::*;
use crate::policy::BridgeRuntimePolicy;

#[test]
fn bridge_harness_subscription_suite_38_to_40_rows_are_present() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let artifact = sealed_phase_18_closeout(BridgeRuntimePolicy::development(), resource_request);
    let rows = artifact.support_matrix().rows();
    assert!(rows
        .iter()
        .any(|row| row.suite_id().as_str() == "suite_38_cost_posture"));
    assert!(rows
        .iter()
        .any(|row| row.suite_id().as_str() == "suite_39_schema_parity"));
    assert!(rows
        .iter()
        .any(|row| row.suite_id().as_str() == "suite_40_multi_failure_precedence"));
}
