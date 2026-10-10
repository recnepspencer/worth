use super::support::*;

#[test]
fn runtime_seals_shared_delivery_bundle_from_ordered_window_and_fanout() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );

    let bundle = shared_delivery_bundle(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let second_bundle = shared_delivery_bundle(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );

    assert_eq!(bundle.digest(), second_bundle.digest());
    assert_eq!(bundle.ordered_causes().len(), 1);
    assert_eq!(
        bundle.ordered_causes()[0].family_kind(),
        crate::facade::BridgeMixedCauseOrderFamilyKind::TruthPatch
    );
    assert_eq!(bundle.consumer_contract_identities().len(), 2);
    assert_eq!(
        bundle
            .counters()
            .subscription_shared_delivery_bundle_sealed_count(),
        1
    );
}

#[test]
fn runtime_rejects_preview_lane_window_for_authoritative_shared_delivery() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let fanout_plan = runtime
        .plan_shared_subscription_fanout(&active, vec![canonical_consumer_contract(&runtime)])
        .expect("fanout plan should admit");
    let fanout_layout = runtime.build_subscription_fanout_layout(
        fanout_plan,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let truth_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-preview"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-preview"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-preview"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-preview"),
    );
    let ordering =
        runtime.order_mixed_causes(&crate::facade::BridgeMixedCauseOrderingRequest::new(
            crate::facade::BridgeMixedCauseOrderingLaneKind::Preview,
            vec![crate::facade::BridgeMixedCauseOrderingInput::TruthPatch(
                truth_patch,
            )],
        ));
    let mixed_window = runtime
        .plan_mixed_cause_delivery_window(
            &ordering,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        )
        .expect("mixed cause window should plan");

    let rejection = runtime
        .plan_shared_subscription_delivery(&active, &mixed_window, &fanout_layout)
        .expect_err("preview lane should reject authoritative shared delivery");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSharedConsumerDeliveryPlanRejectionKind::PreviewLaneRequiresPreviewSurface
    );
}

#[test]
fn runtime_projects_shared_delivery_and_admits_acknowledgement_frontier() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let bundle = shared_delivery_bundle(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );

    let projection = runtime
        .project_shared_delivery_consumer(&bundle, 0)
        .expect("consumer projection should admit");
    let frontier = runtime
        .admit_shared_delivery_acknowledgement_frontier(&bundle, &projection, 0)
        .expect("acknowledgement frontier should admit");

    assert_eq!(projection.consumer_projection_ordinal(), 0);
    assert_eq!(frontier.acknowledged_ordered_cause_sequence(), 0);
    assert_eq!(
        frontier
            .counters()
            .subscription_shared_delivery_acknowledgement_count(),
        1
    );
}

#[test]
fn runtime_rejects_shared_delivery_acknowledgement_for_descriptor_bundle() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let bundle = shared_delivery_bundle(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
    );
    let projection = runtime
        .project_shared_delivery_consumer(&bundle, 0)
        .expect("descriptor projection should still admit");

    let rejection = runtime
        .admit_shared_delivery_acknowledgement_frontier(&bundle, &projection, 0)
        .expect_err("descriptor bundle should reject acknowledgement");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSharedDeliveryAcknowledgementFrontierRejectionKind::DescriptorOnlyFamilyCannotPublishAcknowledgement
    );
}

#[test]
fn runtime_preserves_canonical_bundle_truth_across_sparse_and_coalesced_delivery_posture() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let sparse_bundle = shared_delivery_bundle(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let coalesced_bundle = shared_delivery_bundle(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
    );

    assert_eq!(sparse_bundle.digest(), coalesced_bundle.digest());
    assert_eq!(
        sparse_bundle.ordered_causes(),
        coalesced_bundle.ordered_causes()
    );
    assert_ne!(
        sparse_bundle.delivery_family_identity(),
        coalesced_bundle.delivery_family_identity()
    );
}

#[test]
fn runtime_rejects_shared_delivery_acknowledgement_from_wrong_bundle_projection() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let first_bundle = shared_delivery_bundle(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let fanout_plan = runtime
        .plan_shared_subscription_fanout(&active, vec![canonical_consumer_contract(&runtime)])
        .expect("fanout plan should admit");
    let fanout_layout = runtime.build_subscription_fanout_layout(
        fanout_plan,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let truth_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-b"),
    );
    let ordering =
        runtime.order_mixed_causes(&crate::facade::BridgeMixedCauseOrderingRequest::new(
            crate::facade::BridgeMixedCauseOrderingLaneKind::Authoritative,
            vec![crate::facade::BridgeMixedCauseOrderingInput::TruthPatch(
                truth_patch,
            )],
        ));
    let mixed_window = runtime
        .plan_mixed_cause_delivery_window(
            &ordering,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        )
        .expect("mixed cause window should plan");
    let plan = runtime
        .plan_shared_subscription_delivery(&active, &mixed_window, &fanout_layout)
        .expect("shared delivery plan should admit");
    let layout = runtime.build_shared_subscription_delivery_layout(&plan);
    let draft = runtime.draft_shared_delivery_bundle(&layout);
    let second_bundle = runtime.seal_shared_delivery_bundle(draft);
    let projection = runtime
        .project_shared_delivery_consumer(&first_bundle, 0)
        .expect("projection should admit");

    let rejection = runtime
        .admit_shared_delivery_acknowledgement_frontier(&second_bundle, &projection, 0)
        .expect_err("projection should not acknowledge a different bundle");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSharedDeliveryAcknowledgementFrontierRejectionKind::ProjectionBundleMismatch
    );
}
