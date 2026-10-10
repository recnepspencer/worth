use super::super::support::*;

#[test]
fn runtime_admits_subscription_delivery_cost_profiles() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let sparse = runtime
        .admit_subscription_delivery_cost_profile(
            BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            4,
            1,
            1,
        )
        .expect("sparse profile should admit");
    let coalesced = runtime
        .admit_subscription_delivery_cost_profile(
            BridgeSubscriptionDeliveryDensityPosture::BoundedCoalescedWindow,
            4,
            4,
            1,
        )
        .expect("coalesced profile should admit");

    assert_eq!(
        sparse
            .counters()
            .subscription_delivery_cost_profile_selection_count(),
        1
    );
    assert_eq!(
        sparse
            .counters()
            .subscription_delivery_density_sparse_count(),
        1
    );
    assert_eq!(
        coalesced
            .counters()
            .subscription_delivery_density_coalesced_count(),
        1
    );
}

#[test]
fn runtime_rejects_over_budget_delivery_profile_before_delivery() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let rejection = runtime
        .admit_subscription_delivery_cost_profile(
            BridgeSubscriptionDeliveryDensityPosture::RejectedOverBudget,
            4,
            1,
            1,
        )
        .expect_err("over-budget posture should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionDeliveryCostProfileRejectionKind::OverBudgetPostureRejected
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_delivery_over_budget_rejection_count(),
        1
    );
}

#[test]
fn runtime_rejects_zero_fanout_width_cost_profile_before_activation() {
    let runtime = runtime(BridgeRuntimePolicy::development());

    let rejection = runtime
        .admit_subscription_delivery_cost_profile(
            BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            4,
            1,
            0,
        )
        .expect_err("active subscription needs at least one admitted consumer slot");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionDeliveryCostProfileRejectionKind::EmptyFanoutBudget
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_delivery_cost_profile_rejection_count(),
        1
    );
}

#[test]
fn delivery_window_rejects_member_count_over_cost_profile_before_projection() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        1,
        resource_request,
    );
    let open = runtime.open_subscription_delivery_window(
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
    );

    let rejection = runtime
        .seal_subscription_delivery_window(
            open,
            vec![
                BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
                    "slice:entity-1/profile/name",
                    "routing:fixture:1",
                    BridgeSubscriptionDeliveryMemberClass::Update,
                    BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned(
                        "content:fixture:1",
                    ),
                ),
                BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
                    "slice:entity-1/profile/name",
                    "routing:fixture:2",
                    BridgeSubscriptionDeliveryMemberClass::Update,
                    BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned(
                        "content:fixture:2",
                    ),
                ),
                BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
                    "slice:entity-1/profile/name",
                    "routing:fixture:3",
                    BridgeSubscriptionDeliveryMemberClass::Update,
                    BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned(
                        "content:fixture:3",
                    ),
                ),
                BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
                    "slice:entity-1/profile/name",
                    "routing:fixture:4",
                    BridgeSubscriptionDeliveryMemberClass::Update,
                    BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned(
                        "content:fixture:4",
                    ),
                ),
                BridgeSubscriptionDeliveryMemberInput::delivery_content_digest(
                    "slice:entity-1/profile/name",
                    "routing:fixture:5",
                    BridgeSubscriptionDeliveryMemberClass::Update,
                    BridgeSubscriptionDeliveryContentDigest::admit_bridge_owned(
                        "content:fixture:5",
                    ),
                ),
            ],
        )
        .expect_err("delivery window should reject before constructing records");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionDeliveryWindowRejectionKind::MemberCountExceedsCostProfile
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_delivery_over_budget_rejection_count(),
        1
    );
    assert_eq!(rejection.counters().subscription_delivery_record_count(), 0);
    assert_eq!(
        rejection
            .counters()
            .subscription_rich_diagnostics_hot_path_materialization_count(),
        0
    );
}
