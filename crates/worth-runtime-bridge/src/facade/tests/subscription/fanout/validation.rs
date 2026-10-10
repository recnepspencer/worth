use super::super::support::*;

#[test]
fn fanout_projection_validation_accepts_matching_layout_and_rejects_layout_drift() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let plan = runtime
        .plan_shared_subscription_fanout(&active, vec![canonical_consumer_contract(&runtime)])
        .expect("equivalent consumers should share");
    let layout = runtime.build_subscription_fanout_layout(
        plan,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let sealed = sealed_window(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let projection_set = runtime
        .project_subscription_delivery_to_fanout(&layout, &sealed)
        .expect("projection should match");

    let validation = runtime
        .validate_subscription_fanout_projection(&layout, &projection_set)
        .expect("matching projection set should validate");
    assert_eq!(
        validation
            .counters()
            .subscription_fanout_projection_validation_count(),
        1
    );
    assert_eq!(
        validation
            .counters()
            .subscription_fanout_per_member_consumer_scan_count(),
        0
    );

    let drift_plan = runtime
        .plan_shared_subscription_fanout(&active, vec![canonical_consumer_contract(&runtime)])
        .expect("equivalent consumers should share");
    let drift_layout = runtime.build_subscription_fanout_layout(
        drift_plan,
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
    );
    let rejection = runtime
        .validate_subscription_fanout_projection(&drift_layout, &projection_set)
        .expect_err("different layout identity should reject");
    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionFanoutProjectionValidationRejectionKind::LayoutIdentityMismatch
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_fanout_projection_validation_rejection_count(),
        1
    );
}
