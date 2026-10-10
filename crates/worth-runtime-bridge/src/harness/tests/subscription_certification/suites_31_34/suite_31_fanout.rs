use super::super::support::*;
use crate::facade::{
    BridgeRuntimePolicy, BridgeSubscriptionConsumerBackpressurePosture,
    BridgeSubscriptionConsumerContractFamily, BridgeSubscriptionConsumerDiagnosticsRetention,
    BridgeSubscriptionConsumerPacingCapability, BridgeSubscriptionDeliveryDensityPosture,
    BridgeSubscriptionDeliveryFamilyKind, BridgeSubscriptionDuplicateReplayPolicyKind,
    BridgeSubscriptionFanoutPlanRejectionKind,
};

#[test]
fn bridge_harness_subscription_suite_31_shared_fanout_parity_is_canonical() {
    let host_request = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bridge = runtime(BridgeRuntimePolicy::development());
    let declaration = detail_subscription(&bridge);
    let shared_active = active_subscription_for(
        &bridge,
        &declaration,
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let plan = bridge
        .plan_shared_subscription_fanout(&shared_active, vec![canonical_consumer(&bridge)])
        .expect("equivalent consumers should share one active subscription");
    let layout = bridge.build_subscription_fanout_layout(
        plan,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
    );
    let shared_window = sealed_window_with_members(
        &bridge,
        &shared_active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
        fixture_members(2),
    );
    let projections = bridge
        .project_subscription_delivery_to_fanout(&layout, &shared_window)
        .expect("shared fanout projection should match layout");

    let separate_runtime = runtime(BridgeRuntimePolicy::development());
    let separate_declaration = detail_subscription(&separate_runtime);
    let separate_active = active_subscription_for(
        &separate_runtime,
        &separate_declaration,
        BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
        2,
        resource_request,
    );
    let separate_window = sealed_window_with_members(
        &separate_runtime,
        &separate_active,
        BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        0,
        fixture_members(2),
    );

    assert_eq!(shared_active.digest(), separate_active.digest());
    assert_eq!(layout.consumer_bindings().len(), 2);
    assert_eq!(
        shared_window.members()[0].digest(),
        separate_window.members()[0].digest()
    );
    assert_eq!(
        shared_window.members()[1].digest(),
        separate_window.members()[1].digest()
    );
    assert!(projections
        .canonical_member_digest_basis()
        .contains(shared_window.members()[0].digest()));
    assert!(projections
        .canonical_member_digest_basis()
        .contains(shared_window.members()[1].digest()));
    assert_eq!(projections.len(), 2);
    assert_eq!(
        layout
            .counters()
            .subscription_fanout_per_member_consumer_scan_count(),
        0
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
    let shared_checkpoint = checkpoint_from_sealed(
        &bridge,
        &shared_active,
        &shared_window,
        1,
        BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    let separate_checkpoint = checkpoint_from_sealed(
        &separate_runtime,
        &separate_active,
        &separate_window,
        1,
        BridgeSubscriptionDuplicateReplayPolicyKind::SuppressAcknowledgedMembers,
    );
    assert_eq!(
        shared_checkpoint.checkpoint_identity(),
        separate_checkpoint.checkpoint_identity()
    );
    assert_eq!(shared_checkpoint.digest(), separate_checkpoint.digest());

    let collection_declaration = collection_subscription(&bridge);
    let coalesced_active = active_subscription_for(
        &bridge,
        &collection_declaration,
        BridgeSubscriptionDeliveryDensityPosture::BoundedCoalescedWindow,
        2,
        resource_request,
    );
    let coalesced_plan = bridge
        .plan_shared_subscription_fanout(&coalesced_active, vec![canonical_consumer(&bridge)])
        .expect("equivalent coalescing-admitted consumers should share");
    let coalesced_layout = bridge.build_subscription_fanout_layout(
        coalesced_plan,
        BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
    );
    let coalesced_window = sealed_window_with_members(
        &bridge,
        &coalesced_active,
        BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
        0,
        fixture_members(2),
    );
    let coalesced_projection = bridge
        .project_subscription_delivery_to_fanout(&coalesced_layout, &coalesced_window)
        .expect("admitted coalesced fanout projection should preserve member truth");
    assert!(coalesced_projection
        .canonical_member_digest_basis()
        .contains(coalesced_window.members()[0].digest()));
    assert!(coalesced_projection
        .canonical_member_digest_basis()
        .contains(coalesced_window.members()[1].digest()));
    assert_eq!(
        coalesced_layout
            .counters()
            .subscription_fanout_per_member_consumer_scan_count(),
        0
    );

    let lag_runtime = runtime(BridgeRuntimePolicy::development());
    let lag_declaration = detail_subscription(&lag_runtime);
    let lag_primary = lag_runtime
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            BridgeSubscriptionConsumerPacingCapability::LagBounded,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::MinimalReference,
        )
        .expect("lag-bounded primary consumer should admit");
    let lag_ready = activation_ready_for(&lag_runtime, &lag_declaration, resource_request);
    let lag_cost = lag_runtime
        .admit_subscription_delivery_cost_profile(
            BridgeSubscriptionDeliveryDensityPosture::SparseMemberDelivery,
            4,
            4,
            2,
        )
        .expect("lag fanout cost profile should admit");
    let lag_active = lag_runtime.activate_subscription_delivery(lag_ready, lag_cost, lag_primary);
    let lag_additional = lag_runtime
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            BridgeSubscriptionConsumerPacingCapability::LagBounded,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::MinimalReference,
        )
        .expect("lag-bounded additional consumer should admit");
    let lag_plan = lag_runtime
        .plan_shared_subscription_fanout(&lag_active, vec![lag_additional])
        .expect("equivalent lag-bounded consumers should share");
    assert_eq!(lag_plan.consumer_contract_identity_count(), 2);

    let diagnostics_rich = bridge
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::CanonicalDelivery,
            BridgeSubscriptionConsumerPacingCapability::Immediate,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::RetainedDetail,
        )
        .expect("diagnostics-rich canonical consumer should admit");
    let diagnostics_rejection = bridge
        .plan_shared_subscription_fanout(&shared_active, vec![diagnostics_rich])
        .expect_err("diagnostics-rich consumer must not share minimal fanout silently");
    assert_eq!(
        diagnostics_rejection.rejection_kind(),
        BridgeSubscriptionFanoutPlanRejectionKind::DiagnosticsRetentionMismatch
    );

    let replay_audit = bridge
        .admit_subscription_consumer_contract(
            BridgeSubscriptionConsumerContractFamily::ReplayAudit,
            BridgeSubscriptionConsumerPacingCapability::Immediate,
            BridgeSubscriptionConsumerBackpressurePosture::PacingOnly,
            true,
            BridgeSubscriptionConsumerDiagnosticsRetention::RetainedDetail,
        )
        .expect("replay-audit consumer should admit");
    let rejection = bridge
        .plan_shared_subscription_fanout(&shared_active, vec![replay_audit])
        .expect_err("divergent consumers must reject before delivery");
    assert_eq!(
        rejection.rejection_kind(),
        BridgeSubscriptionFanoutPlanRejectionKind::ContractFamilyMismatch
    );
}
