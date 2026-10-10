use super::super::support::*;

#[test]
fn runtime_admits_consumer_contract_without_callback_identity() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let left = runtime
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            BridgeSubscriptionConsumerPacingCapability::Immediate,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::MinimalReference,
        )
        .expect("consumer contract should admit");
    let right = runtime
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            BridgeSubscriptionConsumerPacingCapability::Immediate,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::MinimalReference,
        )
        .expect("same semantic contract should admit");

    assert_eq!(left, right);
    assert_eq!(
        left.consumer_contract_identity(),
        right.consumer_contract_identity()
    );
    assert_eq!(
        left.counters()
            .subscription_consumer_contract_admission_count(),
        1
    );
    assert_eq!(
        left.sharing_eligibility().digest(),
        right.sharing_eligibility().digest()
    );
}

#[test]
fn runtime_activates_subscription_delivery_from_activation_ready() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, ready) = activation_ready_detail_subscription(resource_request);
    let cost_profile = runtime
        .admit_subscription_delivery_cost_profile(
            BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            4,
            1,
            1,
        )
        .expect("cost profile should admit");
    let consumer = canonical_consumer_contract(&runtime);

    let active = runtime.activate_subscription_delivery(ready, cost_profile, consumer);

    assert_eq!(active.counters().subscription_activation_count(), 1);
    assert_eq!(
        active
            .buffer_plan()
            .counters()
            .subscription_delivery_buffer_reuse_count(),
        1
    );
    assert_eq!(
        active
            .buffer_plan()
            .counters()
            .subscription_delivery_arena_reset_count(),
        1
    );
}

#[test]
fn runtime_emits_stable_canonical_subscription_delivery_records() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );

    let left = sealed_window(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let right = sealed_window(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );

    assert_eq!(left.digest(), right.digest());
    assert_eq!(left.members().len(), 1);
    assert_eq!(left.members()[0].canonical_sequence(), 0);
    assert_eq!(
        left.members()[0].route_or_slice_identity(),
        "slice:entity-1/profile/name"
    );
    assert_eq!(left.counters().subscription_delivery_record_count(), 1);
    assert_eq!(left.counters().subscription_delivery_member_count(), 1);
    assert_eq!(
        left.counters()
            .subscription_rich_diagnostics_hot_path_materialization_count(),
        0
    );
}

#[test]
fn delivery_window_identity_changes_with_canonical_member_truth() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let left_open = runtime.open_subscription_delivery_window(
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
    );
    let right_open = runtime.open_subscription_delivery_window(
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
    );

    let left = runtime
        .seal_subscription_delivery_window(
            left_open,
            vec![
                BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
                    "slice:entity-1/profile/name",
                    "routing:fixture",
                    BridgeSubscriptionDeliveryMemberClass::Update,
                    BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned("content:left"),
                ),
            ],
        )
        .expect("left delivery window should seal");
    let right = runtime
        .seal_subscription_delivery_window(
            right_open,
            vec![
                BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
                    "slice:entity-1/profile/name",
                    "routing:fixture",
                    BridgeSubscriptionDeliveryMemberClass::Update,
                    BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned("content:right"),
                ),
            ],
        )
        .expect("right delivery window should seal");

    assert_ne!(
        left.delivery_window_identity(),
        right.delivery_window_identity()
    );
    assert_ne!(
        left.members()[0].delivery_window_identity(),
        right.members()[0].delivery_window_identity()
    );
}

#[test]
fn delivery_window_identity_changes_with_occurrence_sequence() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let left_open = runtime.open_subscription_delivery_window(
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        41,
    );
    let right_open = runtime.open_subscription_delivery_window(
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        42,
    );

    assert_ne!(
        left_open.delivery_window_open_identity(),
        right_open.delivery_window_open_identity()
    );

    let member = || {
        BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
            "slice:entity-1/profile/name",
            "routing:fixture",
            BridgeSubscriptionDeliveryMemberClass::Update,
            BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned("content:same"),
        )
    };
    let left = runtime
        .seal_subscription_delivery_window(left_open, vec![member()])
        .expect("left delivery window should seal");
    let right = runtime
        .seal_subscription_delivery_window(right_open, vec![member()])
        .expect("right delivery window should seal");

    assert_ne!(
        left.delivery_window_identity(),
        right.delivery_window_identity()
    );
    assert_ne!(left.members()[0].digest(), right.members()[0].digest());
}

#[test]
fn coalesced_delivery_reconstructs_the_same_canonical_member_truth() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::BoundedCoalescedWindow,
        resource_request,
    );

    let canonical = sealed_window(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let coalesced = sealed_window(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
    );

    assert_eq!(canonical.members().len(), coalesced.members().len());
    assert_eq!(
        canonical.members()[0].route_or_slice_identity(),
        coalesced.members()[0].route_or_slice_identity()
    );
    assert_eq!(
        canonical.members()[0].member_class(),
        coalesced.members()[0].member_class()
    );
}
