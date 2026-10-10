use super::support::*;

#[test]
fn resume_admission_accepts_matching_active_subscription_and_plans_next_sequence() {
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
        0,
        fixture_members(2),
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &sealed,
        1,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::RedeliverAcknowledgedMembersWhenIdempotent,
    );

    let admission = runtime
        .admit_subscription_resume(&active, &checkpoint)
        .expect("resume admission should accept matching checkpoint");
    assert_eq!(admission.acknowledged_canonical_sequence(), 1);
    assert_eq!(admission.expected_next_canonical_sequence(), 2);
    assert_eq!(
        admission.counters().subscription_resume_admission_count(),
        1
    );

    let plan = runtime.plan_subscription_resume(admission);
    assert_eq!(plan.resume_after_acknowledged_canonical_sequence(), 1);
    assert_eq!(plan.expected_next_canonical_sequence(), 2);
    assert_eq!(plan.counters().subscription_resume_plan_count(), 1);
}

#[test]
fn resume_admission_rejects_checkpoint_for_different_active_subscription() {
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
        0,
        fixture_members(1),
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &sealed,
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::RedeliverAcknowledgedMembersWhenIdempotent,
    );
    let (_other_runtime, other_active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::BoundedCoalescedWindow,
        resource_request,
    );

    let rejection = runtime
        .admit_subscription_resume(&other_active, &checkpoint)
        .expect_err("checkpoint from different active subscription should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionResumeAdmissionRejectionKind::ActiveSubscriptionMismatch
    );
}

#[test]
fn delivery_replay_plan_admits_ordered_retained_windows_after_checkpoint() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let checkpoint_window = sealed_window_with_members(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
        fixture_members(1),
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &checkpoint_window,
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let admission = runtime
        .admit_subscription_resume(&active, &checkpoint)
        .expect("resume admission should accept matching checkpoint");
    let retained = runtime.retain_subscription_delivery_window_seed(&sealed_window_with_members(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        1,
        fixture_members(2),
    ));

    let plan = runtime
        .plan_subscription_delivery_replay(&active, admission, vec![retained])
        .expect("retained replay should admit windows after checkpoint");

    assert_eq!(plan.retained_window_count(), 1);
    assert_eq!(plan.retained_member_count(), 2);
    assert_eq!(plan.checkpoint_delivery_window_sequence(), 0);
    assert_eq!(plan.counters().subscription_delivery_replay_plan_count(), 1);
}

#[test]
fn delivery_replay_plan_rejects_empty_retained_seed_set() {
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
        &sealed_window_with_members(
            &runtime,
            &active,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
            0,
            fixture_members(1),
        ),
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let admission = runtime
        .admit_subscription_resume(&active, &checkpoint)
        .expect("resume admission should accept matching checkpoint");

    let rejection = runtime
        .plan_subscription_delivery_replay(&active, admission, vec![])
        .expect_err("empty replay seed set should reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionDeliveryReplayPlanRejectionKind::EmptyRetainedWindowSet
    );
}

#[test]
fn delivery_replay_plan_rejects_stale_checkpoint_window_seed() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let checkpoint_window = sealed_window_with_members(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
        fixture_members(1),
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &checkpoint_window,
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let admission = runtime
        .admit_subscription_resume(&active, &checkpoint)
        .expect("resume admission should accept matching checkpoint");
    let retained = runtime.retain_subscription_delivery_window_seed(&checkpoint_window);

    let rejection = runtime
        .plan_subscription_delivery_replay(&active, admission, vec![retained])
        .expect_err("checkpoint window must not be replayed as retained future work");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionDeliveryReplayPlanRejectionKind::RetainedWindowNotAfterCheckpoint
    );
}

#[test]
fn delivery_replay_plan_rejects_ambiguous_duplicate_window_sequences() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let checkpoint_window = sealed_window_with_members(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
        fixture_members(1),
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &checkpoint_window,
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::RejectDuplicateReplay,
    );
    let admission = runtime
        .admit_subscription_resume(&active, &checkpoint)
        .expect("resume admission should accept matching checkpoint");
    let first = runtime.retain_subscription_delivery_window_seed(&sealed_window_with_members(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        1,
        fixture_members(1),
    ));
    let second = runtime.retain_subscription_delivery_window_seed(&sealed_window_with_members(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        1,
        fixture_members(2),
    ));

    let rejection = runtime
        .plan_subscription_delivery_replay(&active, admission, vec![second, first])
        .expect_err("duplicate retained window sequences must reject");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionDeliveryReplayPlanRejectionKind::RetainedWindowSequenceAmbiguous
    );
}

#[test]
fn delivery_replay_plan_rejects_retained_delivery_family_drift() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let checkpoint_window = sealed_window_with_members(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
        fixture_members(1),
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &checkpoint_window,
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let admission = runtime
        .admit_subscription_resume(&active, &checkpoint)
        .expect("resume admission should accept matching checkpoint");
    let retained = runtime.retain_subscription_delivery_window_seed(&sealed_window_with_members(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
        1,
        fixture_members(1),
    ));

    let rejection = runtime
        .plan_subscription_delivery_replay(&active, admission, vec![retained])
        .expect_err("retained family drift must reject before replay construction");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionDeliveryReplayPlanRejectionKind::DeliveryFamilyMismatch
    );
}

#[test]
fn delivery_replay_plan_rejects_retained_window_blocked_by_replay_readiness() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        resource_request,
    );
    let checkpoint_window = sealed_window_with_members(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
        fixture_members(1),
    );
    let checkpoint = checkpoint_from_sealed(
        &runtime,
        &active,
        &checkpoint_window,
        0,
        crate::facade::BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let admission = runtime
        .admit_subscription_resume(&active, &checkpoint)
        .expect("resume admission should accept matching checkpoint");
    let blocked = sealed_window_with_member(
        &runtime,
        &active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        1,
        BridgeSubscriptionDeliveryMemberInput::omitted_content(
            "slice:entity-1/profile/name",
            "routing:fixture",
            BridgeSubscriptionDeliveryMemberClass::Update,
            BridgeSubscriptionDeliveryContentOmissionReason::ContentDigestOnly,
        ),
    );
    let retained = runtime.retain_subscription_delivery_window_seed(&blocked);

    let rejection = runtime
        .plan_subscription_delivery_replay(&active, admission, vec![retained])
        .expect_err("retained windows with blocked readiness must reject replay planning");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionDeliveryReplayPlanRejectionKind::RetainedWindowReplayReadinessBlocked
    );
}
