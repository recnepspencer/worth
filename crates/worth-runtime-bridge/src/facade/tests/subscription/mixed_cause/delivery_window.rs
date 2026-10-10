use super::*;

#[test]
fn runtime_plans_delivery_window_from_ordered_mixed_causes_only() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let truth_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
    );
    let ordering = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![BridgeMixedCauseOrderingInput::TruthPatch(truth_patch)],
    ));

    let window = runtime
        .plan_mixed_cause_delivery_window(
            &ordering,
            BridgeSubscriptionDeliveryFamilyKind::RouteFocusedDescriptor,
        )
        .expect("delivery window should plan");

    assert_eq!(window.ordered_causes().len(), 1);
    assert_eq!(
        window.ordered_causes()[0].family_kind(),
        BridgeMixedCauseOrderFamilyKind::TruthPatch
    );
    assert_eq!(
        window
            .counters()
            .subscription_mixed_cause_delivery_window_plan_count(),
        1
    );
}
