use crate::facade::{
    BridgeRuntimePolicy, BridgeSubscriptionTemporalAsyncCertificationCloseoutArtifact,
    BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest,
};

pub(crate) fn phase_18_closeout_request(
    runtime: &crate::facade::RuntimeBridge,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeSubscriptionTemporalAsyncCertificationCloseoutRequest {
    crate::facade::tests::subscription::support::temporal_async_closeout_request(
        runtime,
        resource_request,
    )
}

pub(crate) fn sealed_phase_18_closeout(
    policy: BridgeRuntimePolicy,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> BridgeSubscriptionTemporalAsyncCertificationCloseoutArtifact {
    let runtime = crate::facade::tests::subscription::support::runtime(policy);
    runtime
        .seal_subscription_temporal_async_certification_closeout(phase_18_closeout_request(
            &runtime,
            resource_request,
        ))
        .expect("phase 18 closeout should seal in harness support")
}
