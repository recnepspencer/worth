use super::super::support::*;

#[test]
fn equivalent_detail_consumers_share_one_active_subscription() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let additional = canonical_consumer_contract(&runtime);

    let plan = runtime
        .plan_shared_subscription_fanout(&active, vec![additional])
        .expect("equivalent consumers should share");

    assert_eq!(
        plan.counters().subscription_fanout_plan_admission_count(),
        1
    );
    assert_eq!(plan.consumer_contract_identity_count(), 2);
    assert_eq!(
        plan.active_subscription_identity(),
        active.active_subscription_identity()
    );
    assert_eq!(
        plan.sharing_eligibility_digest(),
        active.consumer_contract().sharing_eligibility().digest()
    );
}

#[test]
fn equivalent_collection_consumers_share_one_active_subscription() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_collection_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let additional = canonical_consumer_contract(&runtime);

    let plan = runtime
        .plan_shared_subscription_fanout(&active, vec![additional])
        .expect("equivalent collection consumers should share");

    assert_eq!(plan.consumer_contract_identity_count(), 2);
    assert_eq!(
        plan.cost_profile_identity(),
        active.cost_profile().cost_profile_identity()
    );
}

#[test]
fn shared_and_separate_equivalent_consumers_preserve_canonical_delivery_truth() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, shared_active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let additional = canonical_consumer_contract(&runtime);
    let plan = runtime
        .plan_shared_subscription_fanout(&shared_active, vec![additional])
        .expect("equivalent consumers should share");
    let layout = runtime.build_subscription_fanout_layout(
        plan,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let shared_window = sealed_window(
        &runtime,
        &shared_active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let projections = runtime
        .project_subscription_delivery_to_fanout(&layout, &shared_window)
        .expect("projection should match layout");

    let (separate_runtime, separate_active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let separate_window = sealed_window(
        &separate_runtime,
        &separate_active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );

    assert_eq!(
        shared_window.members()[0].digest(),
        separate_window.members()[0].digest()
    );
    assert_eq!(projections.len(), 2);
    assert_eq!(
        projections
            .counters()
            .subscription_fanout_delivery_projection_count(),
        2
    );
    assert!(projections
        .iter()
        .all(|projection| projection.canonical_member_digest_basis()
            == shared_window.members()[0].digest()));
}

#[test]
fn divergent_replay_audit_consumer_rejects_shared_fanout() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let replay_audit = runtime
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::ReplayAudit,
            BridgeSubscriptionConsumerPacingCapability::Immediate,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::RetainedDetail,
        )
        .expect("replay/audit consumer should admit with retained diagnostics");

    let rejection = runtime
        .plan_shared_subscription_fanout(&active, vec![replay_audit])
        .expect_err("replay/audit consumer should not share canonical active delivery");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionFanoutPlanRejectionKind::ContractFamilyMismatch
    );
    assert_eq!(
        rejection
            .counters()
            .subscription_fanout_plan_rejection_count(),
        1
    );
}

#[test]
fn mismatched_coalescing_rejects_shared_fanout() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let non_coalescing = runtime
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            BridgeSubscriptionConsumerPacingCapability::Immediate,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            false,
            BridgeSubscriptionConsumerDiagnosticsRetention::MinimalReference,
        )
        .expect("non-coalescing consumer should admit");

    let rejection = runtime
        .plan_shared_subscription_fanout(&active, vec![non_coalescing])
        .expect_err("coalescing mismatch should reject sharing");

    assert_eq!(
        rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionFanoutPlanRejectionKind::CoalescingMismatch
    );
}

#[test]
fn mismatched_pacing_and_backpressure_reject_shared_fanout() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let (runtime, active) = active_detail_subscription_with_fanout(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let lag_bounded = runtime
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            BridgeSubscriptionConsumerPacingCapability::LagBounded,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::MinimalReference,
        )
        .expect("lag-bounded consumer should admit");

    let pacing_rejection = runtime
        .plan_shared_subscription_fanout(&active, vec![lag_bounded])
        .expect_err("pacing mismatch should reject sharing");

    assert_eq!(
        pacing_rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionFanoutPlanRejectionKind::PacingCapabilityMismatch
    );

    let primary_runtime = super::super::support::runtime(BridgeRuntimePolicy::development());
    let lag_bounded_primary = primary_runtime
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            BridgeSubscriptionConsumerPacingCapability::LagBounded,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::MinimalReference,
        )
        .expect("lag-bounded primary consumer should admit");
    let (runtime, active) = active_detail_subscription_with_consumer(
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        lag_bounded_primary,
        resource_request,
    );
    let independent_cursor = runtime
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            BridgeSubscriptionConsumerPacingCapability::LagBounded,
            BridgeSubscriptionConsumerBackpressurePosture::IndependentCursorRequired,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::MinimalReference,
        )
        .expect("independent cursor consumer should admit with lag-bounded pacing");

    let backpressure_rejection = runtime
        .plan_shared_subscription_fanout(&active, vec![independent_cursor])
        .expect_err("backpressure mismatch should reject sharing");

    assert_eq!(
        backpressure_rejection.rejection_kind(),
        crate::facade::BridgeSubscriptionFanoutPlanRejectionKind::BackpressurePostureMismatch
    );
}
