use super::support::*;

#[test]
fn equivalent_retained_resume_basis_prepares_equal_replay_readiness() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (left_runtime, left_active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let left_checkpoint = checkpoint_from_sealed(
        &left_runtime,
        &left_active,
        &sealed_window_with_members(
            &left_runtime,
            &left_active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
            0,
            fixture_members(2),
        ),
        1,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let left_temporal = retained_temporal_resume_basis(
        &left_runtime,
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        BridgeRetainedTemporalWakePosture::Ready,
        true,
    );
    let left_async = retained_inflight_async_resume_basis(
        &left_runtime,
        &admitted_async_request_identity(
            &left_runtime,
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            7,
        ),
        true,
    );
    let left_retained = retained_subscription_resume_basis(
        &left_runtime,
        &left_active,
        &left_checkpoint,
        Some(left_temporal),
        Some(left_async),
        None,
        true,
    );
    let left_admitted = left_runtime
        .admit_subscription_resume_basis(&left_retained)
        .expect("left resume basis should admit");
    let left_readiness = left_runtime.prepare_subscription_replay_readiness(&left_admitted);

    let (right_runtime, right_active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let right_checkpoint = checkpoint_from_sealed(
        &right_runtime,
        &right_active,
        &sealed_window_with_members(
            &right_runtime,
            &right_active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
            0,
            fixture_members(2),
        ),
        1,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let right_temporal = retained_temporal_resume_basis(
        &right_runtime,
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        BridgeRetainedTemporalWakePosture::Ready,
        true,
    );
    let right_async = retained_inflight_async_resume_basis(
        &right_runtime,
        &admitted_async_request_identity(
            &right_runtime,
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            7,
        ),
        true,
    );
    let right_retained = retained_subscription_resume_basis(
        &right_runtime,
        &right_active,
        &right_checkpoint,
        Some(right_temporal),
        Some(right_async),
        None,
        true,
    );
    let right_admitted = right_runtime
        .admit_subscription_resume_basis(&right_retained)
        .expect("right resume basis should admit");
    let right_readiness = right_runtime.prepare_subscription_replay_readiness(&right_admitted);

    assert_eq!(left_retained.digest(), right_retained.digest());
    assert_eq!(left_admitted.digest(), right_admitted.digest());
    assert_eq!(left_readiness.digest(), right_readiness.digest());
}

#[test]
fn retained_temporal_resume_basis_distinguishes_pending_from_ready_wakes() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let pending = retained_temporal_resume_basis(
        &runtime,
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        BridgeRetainedTemporalWakePosture::Pending,
        true,
    );
    let ready = retained_temporal_resume_basis(
        &runtime,
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        BridgeRetainedTemporalWakePosture::Ready,
        true,
    );

    assert_ne!(pending.digest(), ready.digest());
    assert_eq!(
        pending
            .counters()
            .subscription_resume_temporal_basis_count(),
        1
    );
}

#[test]
fn resume_basis_rejects_missing_inflight_async_generation() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &sealed_window(
            &runtime,
            &active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        ),
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let async_request = admitted_async_request_identity(
        &runtime,
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        9,
    );
    let retained = retained_subscription_resume_basis(
        &runtime,
        &active,
        &checkpoint,
        None,
        Some(retained_inflight_async_resume_basis_without_generation(
            &async_request,
            true,
        )),
        None,
        true,
    );

    let rejection = runtime
        .admit_subscription_resume_basis(&retained)
        .expect_err("missing inflight generation should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionResumeBasisRejectionKind::InflightAsyncGenerationMissing
    );
}

#[test]
fn fanout_checkpoint_requires_explicit_delivery_resume_basis() {
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
    let checkpoint = fanout_checkpoint_from_sealed(
        &runtime,
        &active,
        &sealed_window(
            &runtime,
            &active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        ),
        &fanout_layout,
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let retained =
        retained_subscription_resume_basis(&runtime, &active, &checkpoint, None, None, None, true);

    let rejection = runtime
        .admit_subscription_resume_basis(&retained)
        .expect_err("fanout checkpoint without delivery basis should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionResumeBasisRejectionKind::DeliveryBasisMissing
    );
}

#[test]
fn resume_basis_rejects_cross_branch_temporal_and_async_basis() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &sealed_window(
            &runtime,
            &active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        ),
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let temporal = retained_temporal_resume_basis(
        &runtime,
        crate::truth_identity_fixtures::truth_branch_fixture("truth-left"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        BridgeRetainedTemporalWakePosture::Ready,
        true,
    );
    let inflight_async = retained_inflight_async_resume_basis(
        &runtime,
        &admitted_async_request_identity(
            &runtime,
            crate::truth_identity_fixtures::truth_branch_fixture("truth-right"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
            13,
        ),
        true,
    );
    let retained = retained_subscription_resume_basis(
        &runtime,
        &active,
        &checkpoint,
        Some(temporal),
        Some(inflight_async),
        None,
        true,
    );

    let rejection = runtime
        .admit_subscription_resume_basis(&retained)
        .expect_err("cross-branch resume basis should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionResumeBasisRejectionKind::CrossBranchResumeRejected
    );
}

#[test]
fn replay_readiness_carries_shared_delivery_acknowledgement_frontier() {
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
    let checkpoint = fanout_checkpoint_from_sealed(
        &runtime,
        &active,
        &sealed_window_with_members(
            &runtime,
            &active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
            0,
            fixture_members(2),
        ),
        &fanout_layout,
        1,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let bundle = shared_delivery_bundle(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let (_projection, _acknowledgement, delivery_basis) =
        retained_shared_delivery_resume_basis(&runtime, &bundle);
    let retained = retained_subscription_resume_basis(
        &runtime,
        &active,
        &checkpoint,
        None,
        None,
        Some(delivery_basis),
        true,
    );
    let admitted = runtime
        .admit_subscription_resume_basis(&retained)
        .expect("retained basis should admit");
    let readiness = runtime.prepare_subscription_replay_readiness(&admitted);

    assert_eq!(readiness.expected_next_canonical_sequence(), 2);
    assert_eq!(readiness.acknowledged_ordered_cause_sequence(), Some(0));
    assert_eq!(
        readiness
            .counters()
            .subscription_resume_replay_readiness_count(),
        1
    );
}

#[test]
fn retained_resume_basis_lowers_into_existing_replay_resume_admission() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let sealed = sealed_window_with_members(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        2,
        fixture_members(2),
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &sealed,
        1,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let retained =
        retained_subscription_resume_basis(&runtime, &active, &checkpoint, None, None, None, true);
    let admitted_basis = runtime
        .admit_subscription_resume_basis(&retained)
        .expect("retained resume basis should admit");
    let readiness = runtime.prepare_subscription_replay_readiness(&admitted_basis);
    let resumed = runtime
        .admit_subscription_resume_from_basis(&active, &admitted_basis, &readiness)
        .expect("replay readiness should lower into resume admission");
    let direct = runtime
        .admit_subscription_resume(&active, &checkpoint)
        .expect("direct checkpoint resume should admit");

    assert_eq!(resumed.digest(), direct.digest());
    assert_eq!(
        resumed.expected_next_canonical_sequence(),
        readiness.expected_next_canonical_sequence()
    );
}
