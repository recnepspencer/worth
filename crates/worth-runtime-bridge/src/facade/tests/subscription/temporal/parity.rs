use super::super::support::*;

#[test]
fn runtime_replays_equal_temporal_inputs_to_equal_cause_digests() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let left_temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            )),
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("left temporal subscription should admit");
    let right_temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            )),
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("right temporal subscription should admit");
    let left_ready = runtime.prepare_temporal_subscription_activation(&left_temporal);
    let right_ready = runtime.prepare_temporal_subscription_activation(&right_temporal);
    let left_request = runtime.prepare_temporal_wake_routing(&left_ready);
    let right_request = runtime.prepare_temporal_wake_routing(&right_ready);

    let left = runtime
        .route_temporal_wake(&left_request, None)
        .expect("left cause should route");
    let right = runtime
        .route_temporal_wake(&right_request, None)
        .expect("right cause should route");

    assert_eq!(left.digest(), right.digest());
    assert_eq!(left.cause_record_identity(), right.cause_record_identity());
}

#[test]
fn runtime_keeps_truth_plus_time_routing_and_delivery_plan_invariant_to_prior_time_only_order() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);

    let earlier_temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            admitted_temporal_basis_with_wake(
                BridgeTemporalTruthViewBasis::authoritative(
                    crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
                31,
                8,
                8,
            ),
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("earlier temporal subscription should admit");
    let earlier_request = runtime.prepare_temporal_wake_routing(
        &runtime.prepare_temporal_subscription_activation(&earlier_temporal),
    );
    let earlier_cause = runtime
        .route_temporal_wake(&earlier_request, None)
        .expect("earlier time-only cause should route");

    let later_temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            admitted_temporal_basis_with_wake(
                BridgeTemporalTruthViewBasis::authoritative(
                    crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                    crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
                    crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                ),
                32,
                9,
                9,
            ),
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("later temporal subscription should admit");
    let later_request = runtime.prepare_temporal_wake_routing(
        &runtime.prepare_temporal_subscription_activation(&later_temporal),
    );
    let truth_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
    );

    let routed_after_prior = runtime
        .route_temporal_wake_with_truth_patch(&later_request, &truth_patch, Some(&earlier_cause))
        .expect("later truth-plus-time cause should route after earlier cause");
    let routed_without_prior = runtime
        .route_temporal_wake_with_truth_patch(&later_request, &truth_patch, None)
        .expect("later truth-plus-time cause should route without prior cause");
    let planned_after_prior = runtime.plan_temporal_delivery_window(
        &routed_after_prior,
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
    );
    let planned_without_prior = runtime.plan_temporal_delivery_window(
        &routed_without_prior,
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
    );

    assert_eq!(routed_after_prior.digest(), routed_without_prior.digest());
    assert_eq!(planned_after_prior.digest(), planned_without_prior.digest());
}
