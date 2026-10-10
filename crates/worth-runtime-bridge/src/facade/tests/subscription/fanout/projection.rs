use super::super::support::*;

#[test]
fn fanout_width_over_cost_profile_rejects_before_layout() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let additional = canonical_consumer_contract(&runtime);

    let rejection = runtime
        .plan_shared_subscription_fanout(&active, vec![additional])
        .expect_err("cost profile max fanout width is one");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionFanoutPlanRejectionKind::FanoutWidthExceedsCostProfile
    );
}

#[test]
fn fanout_layout_binds_ordered_consumer_slots_deterministically() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        3,
        resource_request,
    );
    let second = canonical_consumer_contract(&runtime);
    let third = canonical_consumer_contract(&runtime);
    let plan = runtime
        .plan_shared_subscription_fanout(&active, vec![second, third])
        .expect("equivalent consumers should share");

    let left = runtime.build_subscription_fanout_layout(
        plan.clone(),
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let right = runtime.build_subscription_fanout_layout(
        plan,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );

    assert_eq!(left.digest(), right.digest());
    assert_eq!(left.consumer_bindings().len(), 3);
    assert_eq!(left.consumer_bindings()[0].slot_index(), 0);
    assert_eq!(left.consumer_bindings()[1].slot_index(), 1);
    assert_eq!(left.consumer_bindings()[2].slot_index(), 2);
    assert_eq!(left.consumer_bindings()[0].frontier_slot_index(), 0);
    assert_eq!(left.consumer_bindings()[1].frontier_slot_index(), 1);
    assert_eq!(left.consumer_bindings()[2].frontier_slot_index(), 2);
    assert_eq!(
        left.consumer_bindings()[0].acknowledgement_policy_class(),
        crate::facade::BridgeSubscriptionFanoutAcknowledgementPolicyClass::CanonicalMemberAcknowledgement
    );
    assert_eq!(
        left.consumer_bindings()[0].diagnostics_policy_class(),
        crate::facade::BridgeSubscriptionFanoutDiagnosticsPolicyClass::MinimalReferenceOnly
    );
    assert_eq!(left.counters().subscription_fanout_layout_build_count(), 1);
    assert_eq!(
        left.counters().subscription_fanout_consumer_binding_count(),
        3
    );
}

#[test]
fn delivery_projection_preserves_canonical_truth_without_rich_materialization() {
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

    let projections = runtime
        .project_subscription_delivery_to_fanout(&layout, &sealed)
        .expect("sealed window should project through matching layout");

    assert_eq!(projections.len(), 2);
    assert_eq!(
        projections.canonical_member_digest_basis(),
        sealed.members()[0].digest()
    );
    assert_eq!(
        projections
            .counters()
            .subscription_fanout_delivery_projection_count(),
        2
    );
    for projection in projections.iter() {
        assert_eq!(
            projection.delivery_window_identity(),
            sealed.delivery_window_identity()
        );
        assert_eq!(projection.canonical_member_count(), sealed.members().len());
        assert_eq!(
            projection.canonical_member_digest_basis(),
            sealed.members()[0].digest()
        );
        assert_eq!(
            projection
                .counters()
                .subscription_fanout_delivery_projection_count(),
            0
        );
        assert_eq!(
            projection
                .counters()
                .subscription_rich_diagnostics_hot_path_materialization_count(),
            0
        );
    }
}

#[test]
fn fanout_counters_prove_zero_hot_path_scans() {
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

    assert_eq!(
        layout
            .counters()
            .subscription_callback_identity_scan_count(),
        0
    );
    assert_eq!(
        layout.counters().subscription_active_registry_scan_count(),
        0
    );
    assert_eq!(
        layout
            .counters()
            .subscription_fanout_per_member_consumer_scan_count(),
        0
    );
}

#[test]
fn fanout_projection_rejects_delivery_family_drift() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::BoundedCoalescedWindow,
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
    let coalesced = sealed_window(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
    );

    let rejection = runtime
        .project_subscription_delivery_to_fanout(&layout, &coalesced)
        .expect_err("family drift should reject projection");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionFanoutProjectionRejectionKind::DeliveryFamilyMismatch
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_fanout_delivery_projection_rejection_count(),
        1
    );
}
