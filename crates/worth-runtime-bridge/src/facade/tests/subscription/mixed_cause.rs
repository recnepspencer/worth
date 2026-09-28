use super::support::*;
use crate::facade::tests::source::support::{
    admit_request_response_completion, denied_request_response_completion_with_displacing_identity,
    request_response_revalidation_lineage,
};
use crate::facade::{
    BridgeAsyncCompletionSupersessionClassificationRequest, BridgeAsyncRequestTruthViewBasis,
    BridgeMixedCauseAsyncResultCause, BridgeMixedCauseAsyncResultDisposition,
    BridgeMixedCauseComparisonReasonKind, BridgeMixedCauseDeniedKind,
    BridgeMixedCauseOrderFamilyKind, BridgeMixedCauseOrdering, BridgeMixedCauseOrderingInput,
    BridgeMixedCauseOrderingLaneKind, BridgeMixedCauseOrderingRequest,
    BridgeTemporalSubscriptionFamilyKind, RuntimeBridge,
};
use worth_signal::facade::NodeId;

mod async_result;

#[test]
fn runtime_orders_mixed_causes_canonically_across_shuffled_input_order() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let truth_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
    );
    let truth_plus_time = authoritative_truth_plus_time_cause(&runtime, &truth_patch);
    let time_only = authoritative_time_only_cause(&runtime);
    let async_completion = admit_request_response_completion(
        &runtime,
        NodeId::new(241, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        64,
    )
    .admitted_completion()
    .expect("completion should admit")
    .clone();

    let first = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::AsyncCompletion(async_completion.clone()),
            BridgeMixedCauseOrderingInput::Temporal(time_only.clone()),
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch.clone()),
            BridgeMixedCauseOrderingInput::Temporal(truth_plus_time.clone()),
        ],
    ));
    let second = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::Temporal(truth_plus_time),
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch),
            BridgeMixedCauseOrderingInput::AsyncCompletion(async_completion),
            BridgeMixedCauseOrderingInput::Temporal(time_only),
        ],
    ));

    assert_eq!(first.digest(), second.digest());
    assert_eq!(
        first
            .ordered()
            .iter()
            .map(|entry| entry.family_kind())
            .collect::<Vec<_>>(),
        vec![
            BridgeMixedCauseOrderFamilyKind::TruthPatch,
            BridgeMixedCauseOrderFamilyKind::TemporalTruthPlusTime,
            BridgeMixedCauseOrderFamilyKind::AsyncCompletion,
            BridgeMixedCauseOrderFamilyKind::TemporalTimeOnly,
        ]
    );
    assert_eq!(
        first.ordered()[0].comparison_evidence().reason_kind(),
        BridgeMixedCauseComparisonReasonKind::RootAdmission
    );
    assert_eq!(
        first.ordered()[1].comparison_evidence().reason_kind(),
        BridgeMixedCauseComparisonReasonKind::PriorityClass
    );
    assert!(first.suppressed().is_empty());
    assert!(first.denied().is_empty());
    let transition = first
        .async_result_transitions()
        .first()
        .expect("ordered completion should retain one async transition");
    assert_eq!(
        transition.disposition(),
        BridgeMixedCauseAsyncResultDisposition::Ordered { ordinal: 2 }
    );
    assert!(matches!(
        transition.cause(),
        BridgeMixedCauseAsyncResultCause::Completion(_)
    ));
    assert_eq!(
        first.counters().subscription_mixed_cause_ordering_count(),
        1
    );
}

#[test]
fn runtime_suppresses_duplicate_mixed_cause_digests_explicitly() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let truth_patch = committed_patch(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_patch_fixture("patch-a"),
    );

    let ordering = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch.clone()),
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch),
        ],
    ));

    assert_eq!(ordering.ordered().len(), 1);
    assert_eq!(ordering.suppressed().len(), 1);
    assert_eq!(
        ordering
            .suppressed()
            .first()
            .expect("suppression should exist")
            .suppressed_kind(),
        crate::facade::BridgeMixedCauseSuppressedKind::DuplicateDigest
    );
    assert_eq!(
        ordering.suppressed()[0].comparison_evidence().reason_kind(),
        BridgeMixedCauseComparisonReasonKind::DuplicateDigestSuppression
    );

    let async_completion = admit_request_response_completion(
        &runtime,
        NodeId::new(244, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        64,
    )
    .admitted_completion()
    .expect("completion should admit")
    .clone();
    let async_ordering = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::AsyncCompletion(async_completion.clone()),
            BridgeMixedCauseOrderingInput::AsyncCompletion(async_completion),
        ],
    ));
    assert_eq!(async_ordering.async_result_transitions().len(), 2);
    assert_eq!(
        async_ordering.async_result_transitions()[0].disposition(),
        BridgeMixedCauseAsyncResultDisposition::Ordered { ordinal: 0 }
    );
    assert_eq!(
        async_ordering.async_result_transitions()[1].disposition(),
        BridgeMixedCauseAsyncResultDisposition::DuplicateSuppressed
    );
}

#[test]
fn runtime_denies_preview_local_causes_in_authoritative_mixed_window() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let admitted = admitted_detail_subscription_in_runtime(&runtime);
    let preview_basis = admitted_preview_basis_for_truth(
        &runtime,
        "mixed-preview",
        crate::truth_identity_fixtures::truth_branch_fixture("truth-preview"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let preview_temporal_basis =
        admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-preview"),
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
        .expect("preview temporal should admit");
    let preview_ready = runtime.prepare_preview_temporal_subscription_activation(&preview_temporal);
    let preview_request = runtime.prepare_preview_temporal_wake_routing(&preview_ready);
    let preview_cause = runtime
        .route_temporal_wake(&preview_request, None)
        .expect("preview wake should route");

    let ordering = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![BridgeMixedCauseOrderingInput::Temporal(preview_cause)],
    ));

    assert!(ordering.ordered().is_empty());
    assert_eq!(ordering.denied().len(), 1);
    assert_eq!(
        ordering.denied()[0].denied_kind(),
        BridgeMixedCauseDeniedKind::AuthoritativePreviewCauseRejected
    );
    assert_eq!(
        ordering.denied()[0].comparison_evidence().reason_kind(),
        BridgeMixedCauseComparisonReasonKind::AuthoritativePreviewRejection
    );
}

#[test]
fn runtime_denies_stale_async_causes_and_non_deliverable_lineage() {
    let runtime = runtime(BridgeRuntimePolicy::development());
    let inputs = stale_and_revalidation_inputs(&runtime);
    let ordering = runtime.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        inputs,
    ));

    assert!(ordering.ordered().is_empty());
    assert_eq!(ordering.denied().len(), 2);
    assert_eq!(
        ordering.denied()[0].denied_kind(),
        BridgeMixedCauseDeniedKind::AsyncStaleCauseRejected
    );
    assert_eq!(
        ordering.denied()[1].denied_kind(),
        BridgeMixedCauseDeniedKind::AsyncLineageNonDeliverable
    );
    assert_eq!(
        ordering.denied()[0].comparison_evidence().reason_kind(),
        BridgeMixedCauseComparisonReasonKind::AsyncStaleDenial
    );
    assert_eq!(
        ordering.denied()[1].comparison_evidence().reason_kind(),
        BridgeMixedCauseComparisonReasonKind::AsyncLineageNonDeliverable
    );
    assert_denied_async_transitions(&ordering);
}

fn stale_and_revalidation_inputs(runtime: &RuntimeBridge) -> Vec<BridgeMixedCauseOrderingInput> {
    let original_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    );
    let current_basis = BridgeAsyncRequestTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-b"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-b"),
    );
    let (denied, displacing_request) = denied_request_response_completion_with_displacing_identity(
        runtime,
        NodeId::new(242, 0),
        original_basis,
        current_basis.clone(),
    );
    let classified = runtime
        .classify_async_completion_supersession(
            BridgeAsyncCompletionSupersessionClassificationRequest::request_response(
                &denied,
                current_basis.clone(),
            )
            .with_displacing_request_identity(&displacing_request),
        )
        .expect("classified denied completion should admit");
    let lineage = request_response_revalidation_lineage(
        runtime,
        NodeId::new(243, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
            crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
        ),
        current_basis,
    );
    vec![
        BridgeMixedCauseOrderingInput::AsyncClassifiedDeniedCompletion(classified),
        BridgeMixedCauseOrderingInput::AsyncRevalidationLineage(lineage),
    ]
}

fn assert_denied_async_transitions(ordering: &BridgeMixedCauseOrdering) {
    assert_eq!(ordering.async_result_transitions().len(), 2);
    assert_eq!(
        ordering.async_result_transitions()[0].disposition(),
        BridgeMixedCauseAsyncResultDisposition::DeliveryDenied(
            BridgeMixedCauseDeniedKind::AsyncStaleCauseRejected
        )
    );
    assert!(matches!(
        ordering.async_result_transitions()[0].cause(),
        BridgeMixedCauseAsyncResultCause::ClassifiedDenied { .. }
    ));
    assert_eq!(
        ordering.async_result_transitions()[1].disposition(),
        BridgeMixedCauseAsyncResultDisposition::DeliveryDenied(
            BridgeMixedCauseDeniedKind::AsyncLineageNonDeliverable
        )
    );
    assert!(matches!(
        ordering.async_result_transitions()[1].cause(),
        BridgeMixedCauseAsyncResultCause::Revalidation(_)
    ));
}

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

fn authoritative_time_only_cause(
    runtime: &crate::facade::RuntimeBridge,
) -> crate::facade::BridgeTemporalCauseRecord {
    let admitted = admitted_detail_subscription_in_runtime(runtime);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("temporal should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let request = runtime.prepare_temporal_wake_routing(&ready);
    runtime
        .route_temporal_wake(&request, None)
        .expect("time-only cause should route")
}

fn authoritative_truth_plus_time_cause(
    runtime: &crate::facade::RuntimeBridge,
    truth_patch: &crate::facade::BridgeCommittedPatchEnvelope,
) -> crate::facade::BridgeTemporalCauseRecord {
    let admitted = admitted_detail_subscription_in_runtime(runtime);
    let temporal_basis = admitted_temporal_basis(BridgeTemporalTruthViewBasis::authoritative(
        crate::truth_identity_fixtures::truth_branch_fixture("truth-main"),
        crate::truth_identity_fixtures::truth_commit_fixture("commit-a"),
        crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
    ));
    let temporal = runtime
        .admit_temporal_subscription(
            &admitted,
            temporal_basis,
            BridgeTemporalSubscriptionFamilyKind::WakeDriven,
        )
        .expect("temporal should admit");
    let ready = runtime.prepare_temporal_subscription_activation(&temporal);
    let request = runtime.prepare_temporal_wake_routing(&ready);
    runtime
        .route_temporal_wake_with_truth_patch(&request, truth_patch, None)
        .expect("truth-plus-time cause should route")
}
