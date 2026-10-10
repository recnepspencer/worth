use super::super::support::*;
use crate::facade::{
    BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId,
    BridgeSubscriptionTemporalAsyncCertificationSupportMatrixVerdict,
};
use crate::policy::BridgeRuntimePolicy;

#[test]
fn bridge_harness_subscription_suite_47_to_50_close_the_milestone() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let artifact = sealed_phase_18_closeout(BridgeRuntimePolicy::development(), resource_request);
    let rows = artifact.support_matrix().rows();
    assert!(rows.iter().any(|row| row.suite_id()
        == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite47DeniedContinuation));
    assert!(rows.iter().any(|row| {
        row.suite_id()
            == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite48TemporalAsyncBundleParity
            && row.verdict()
                == BridgeSubscriptionTemporalAsyncCertificationSupportMatrixVerdict::ParityBandProven
    }));
    assert!(rows.iter().any(|row| row.suite_id() == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite49ReferenceWorkloadSufficiency));
    assert!(rows.iter().any(|row| row.suite_id()
        == BridgeSubscriptionTemporalAsyncCertificationCloseoutSuiteId::Suite50MergedCloseout));
    assert_eq!(artifact.support_matrix().rows().len(), 13);
}
