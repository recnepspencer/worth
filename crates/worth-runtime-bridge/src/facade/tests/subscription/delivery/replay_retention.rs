use super::super::support::*;

#[test]
fn retained_delivery_seed_binds_window_sequence_and_member_truth() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );

    let first = sealed_window_with_member(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        7,
        BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
            "slice:entity-1/profile/name",
            "routing:fixture",
            BridgeSubscriptionDeliveryMemberClass::Update,
            BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned("content:same"),
        ),
    );
    let second_sequence = sealed_window_with_member(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        8,
        BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
            "slice:entity-1/profile/name",
            "routing:fixture",
            BridgeSubscriptionDeliveryMemberClass::Update,
            BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned("content:same"),
        ),
    );
    let second_truth = sealed_window_with_member(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        7,
        BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
            "slice:entity-1/profile/name",
            "routing:fixture",
            BridgeSubscriptionDeliveryMemberClass::Update,
            BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned("content:different"),
        ),
    );

    let first_seed = runtime.retain_subscription_delivery_window_seed(&first);
    let second_sequence_seed = runtime.retain_subscription_delivery_window_seed(&second_sequence);
    let second_truth_seed = runtime.retain_subscription_delivery_window_seed(&second_truth);

    assert_eq!(
        first_seed.delivery_window_identity(),
        first.delivery_window_identity()
    );
    assert_eq!(
        first_seed.active_subscription_identity(),
        first.active_subscription_identity()
    );
    assert_eq!(
        first_seed.admitted_subscription_identity(),
        first.admitted_subscription_identity()
    );
    assert_eq!(first_seed.basis_identity(), first.basis_identity());
    assert_eq!(first_seed.delivery_window_sequence(), 7);
    assert_eq!(
        first_seed.canonical_member_digest_basis(),
        first.members()[0].digest()
    );
    assert_eq!(
        first_seed.replay_readiness_class(),
        crate::facade::BridgeSubscriptionDeliveryReplayReadinessClass::CanonicalMemberReplayReady
    );
    assert_ne!(first_seed.digest(), second_sequence_seed.digest());
    assert_ne!(first_seed.digest(), second_truth_seed.digest());
    assert_eq!(
        first_seed
            .counters()
            .subscription_delivery_window_seed_retention_count(),
        1
    );
}

#[test]
fn replay_readiness_blocks_omitted_content_for_canonical_replay() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let sealed = sealed_window_with_member(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
        BridgeSubscriptionDeliveryMemberInput::omitted_content(
            "slice:entity-1/profile/name",
            "routing:fixture",
            BridgeSubscriptionDeliveryMemberClass::Update,
            BridgeSubscriptionDeliveryContentOmissionReason::ContentDigestOnly,
        ),
    );

    let readiness = runtime.inspect_subscription_delivery_replay_readiness(&sealed);

    assert_eq!(
        readiness.readiness_class(),
        crate::facade::BridgeSubscriptionDeliveryReplayReadinessClass::ReplayBlockedByOmittedContent
    );
    assert_eq!(
        readiness
            .counters()
            .subscription_delivery_replay_readiness_inspection_count(),
        1
    );
    assert_eq!(
        readiness
            .counters()
            .subscription_rich_diagnostics_hot_path_materialization_count(),
        0
    );
}

#[test]
fn descriptor_replay_readiness_ignores_content_omission() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let sealed = sealed_window_with_member(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
        0,
        BridgeSubscriptionDeliveryMemberInput::omitted_content(
            "route:entity-1",
            "routing:fixture",
            BridgeSubscriptionDeliveryMemberClass::Update,
            BridgeSubscriptionDeliveryContentOmissionReason::RouteFocusedDelivery,
        ),
    );

    let readiness = runtime.inspect_subscription_delivery_replay_readiness(&sealed);

    assert_eq!(
        readiness.readiness_class(),
        crate::facade::BridgeSubscriptionDeliveryReplayReadinessClass::DescriptorOnlyReplayReady
    );
}

#[test]
fn descriptor_family_retains_descriptor_replay_seed_without_reconstruction() {
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
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
    );
    let sealed = sealed_window(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
    );
    let projection_set = runtime
        .project_subscription_delivery_to_fanout(&layout, &sealed)
        .expect("descriptor projection should match layout");

    let replay_seed = runtime.retain_subscription_fanout_projection_seed(&projection_set);
    let readiness = runtime.inspect_subscription_delivery_replay_readiness(&sealed);

    assert_eq!(
        readiness.readiness_class(),
        crate::facade::BridgeSubscriptionDeliveryReplayReadinessClass::DescriptorOnlyReplayReady
    );
    assert_eq!(
        replay_seed.canonical_member_digest_basis(),
        projection_set.canonical_member_digest_basis()
    );
    assert_eq!(
        replay_seed.fanout_projection_set_identity(),
        projection_set.fanout_delivery_projection_set_identity()
    );
    assert_eq!(
        replay_seed.delivery_window_identity(),
        projection_set.delivery_window_identity()
    );
    assert_eq!(
        replay_seed
            .counters()
            .subscription_delivery_replay_seed_retention_count(),
        1
    );
}
