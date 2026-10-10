use super::super::support::*;
use crate::facade::{
    BridgeSubscriptionCertificationFailureBoundary,
    BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId,
};
use crate::policy::BridgeRuntimePolicy;

#[test]
fn bridge_harness_subscription_suite_44_to_46_are_typed_rejections() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let artifact = sealed_phase_18_closeout(BridgeRuntimePolicy::development(), resource_request);
    let rows = artifact.support_matrix().rows();
    let unsupported_basis = rows
        .iter()
        .find(|row| row.suite_id() == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite44UnsupportedBasis)
        .expect("suite 44 row should exist");
    let unsupported_neighbor = rows
        .iter()
        .find(|row| row.suite_id() == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite46UnsupportedNeighbor)
        .expect("suite 46 row should exist");

    assert_eq!(
        unsupported_basis.primary_failure_boundary(),
        Some(BridgeSubscriptionCertificationFailureBoundary::BasisDrift)
    );
    assert_eq!(
        unsupported_neighbor.primary_failure_boundary(),
        Some(BridgeSubscriptionCertificationFailureBoundary::IllegalSharingReuse)
    );
}
