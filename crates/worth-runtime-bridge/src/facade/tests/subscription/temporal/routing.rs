use super::super::support::*;

#[test]
fn runtime_routes_time_only_temporal_cause_for_authoritative_lane() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("temporal subscription should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let request = runtime.prepare_temporal_wake_routing(&ready);

    let cause = runtime
        .route_temporal_wake(&request, None)
        .expect("time-only routing should admit");
    let plan = runtime.plan_temporal_delivery_window(
        &cause,
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
    );

    assert_eq!(
        cause.classification(),
        crate::facade::BridgeTemporalCauseClassification::TimeOnly
    );
    assert_eq!(
        cause.routing_lane_kind(),
        crate::facade::BridgeTemporalRoutingLaneKind::Authoritative
    );
    assert!(cause.truth_patch_identity().is_none());
    assert_eq!(
        cause
            .counters()
            .subscription_temporal_time_only_cause_count(),
        1
    );
    assert_eq!(
        plan.counters().subscription_temporal_delivery_plan_count(),
        1
    );
}

#[test]
fn runtime_routes_truth_plus_time_temporal_cause_for_authoritative_lane() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("temporal subscription should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let request = runtime.prepare_temporal_wake_routing(&ready);
    let truth_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
    );

    let cause = runtime
        .route_temporal_wake_with_truth_patch(&request, &truth_patch, None)
        .expect("truth-plus-time routing should admit");

    assert_eq!(
        cause.classification(),
        crate::facade::BridgeTemporalCauseClassification::TruthPlusTime
    );
    assert_eq!(
        cause.truth_patch_identity(),
        Some(truth_patch.patch_identity())
    );
    assert_eq!(
        cause
            .counters()
            .subscription_temporal_truth_plus_time_cause_count(),
        1
    );
}

#[test]
fn runtime_rejects_duplicate_temporal_wake_submission() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("temporal subscription should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let request = runtime.prepare_temporal_wake_routing(&ready);
    let first = runtime
        .route_temporal_wake(&request, None)
        .expect("first wake should route");

    let rejection = runtime
        .route_temporal_wake(&request, Some(&first))
        .expect_err("duplicate wake routing should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeTemporalWakeRoutingRejectionKind::DuplicateWakeSubmission
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_temporal_duplicate_clock_rejection_count(),
        1
    );
}

#[test]
fn runtime_rejects_stale_temporal_wake_submission() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let later_basis = admitted_temporal_basis_with_wake(
        BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        21,
        8,
        9,
    );
    let later_temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            later_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("later temporal subscription should admit");
    let later_ready = runtime.prepare_temporal_subscription_activation(&later_temporal);
    let later_request = runtime.prepare_temporal_wake_routing(&later_ready);
    let later_cause = runtime
        .route_temporal_wake(&later_request, None)
        .expect("later cause should route");

    let earlier_basis = admitted_temporal_basis_with_wake(
        BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        20,
        7,
        8,
    );
    let earlier_temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            earlier_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("earlier temporal subscription should admit");
    let earlier_ready = runtime.prepare_temporal_subscription_activation(&earlier_temporal);
    let earlier_request = runtime.prepare_temporal_wake_routing(&earlier_ready);

    let rejection = runtime
        .route_temporal_wake(&earlier_request, Some(&later_cause))
        .expect_err("earlier wake should reject as stale");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeTemporalWakeRoutingRejectionKind::StaleWakeSubmission
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_temporal_stale_clock_rejection_count(),
        1
    );
}

#[test]
fn runtime_rejects_temporal_truth_plus_time_patch_snapshot_drift() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("temporal subscription should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let request = runtime.prepare_temporal_wake_routing(&ready);
    let truth_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
    );

    let rejection = runtime
        .route_temporal_wake_with_truth_patch(&request, &truth_patch, None)
        .expect_err("snapshot-drift patch should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeTemporalWakeRoutingRejectionKind::TruthPatchSnapshotIdentityMismatch
    );
}

#[test]
fn runtime_rejects_temporal_truth_plus_time_patch_branch_drift() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("temporal subscription should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let request = runtime.prepare_temporal_wake_routing(&ready);
    let truth_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("analysis-drift"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
    );

    let rejection = runtime
        .route_temporal_wake_with_truth_patch(&request, &truth_patch, None)
        .expect_err("branch-drift patch should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeTemporalWakeRoutingRejectionKind::TruthPatchBranchIdentityMismatch
    );
}

#[test]
fn runtime_routes_preview_temporal_wake_and_rejects_lane_mismatch_prior_cause() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime, resource_request);
    let authoritative_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-branch:preview"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-preview"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            authoritative_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("authoritative temporal subscription should admit");
    let authoritative_ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let authoritative_request = runtime.prepare_temporal_wake_routing(&authoritative_ready);
    let authoritative_cause = runtime
        .route_temporal_wake(&authoritative_request, None)
        .expect("authoritative cause should route");

    let preview_basis = admitted_preview_basis_for_truth(
        &runtime,
        "preview",
        crate::truth_identity_fixtures::truth_branch_fixture("truth-branch:preview"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let preview_temporal_basis =
        admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-branch:preview"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-preview"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ));
    let preview_temporal = runtime
        .admit_preview_temporal_subscription(
            &admitted,
            &preview_basis,
            preview_temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("preview temporal subscription should admit");
    let preview_ready = runtime.prepare_preview_temporal_subscription_activation(&preview_temporal);
    let preview_request = runtime.prepare_preview_temporal_wake_routing(&preview_ready);

    let preview_cause = runtime
        .route_temporal_wake(&preview_request, None)
        .expect("preview wake should route");
    let rejection = runtime
        .route_temporal_wake(&preview_request, Some(&authoritative_cause))
        .expect_err("authoritative prior cause should not satisfy preview lane");

    assert_eq!(
        preview_cause.routing_lane_kind(),
        crate::facade::BridgeTemporalRoutingLaneKind::Preview
    );
    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeTemporalWakeRoutingRejectionKind::RoutingLaneMismatch
    );
}
