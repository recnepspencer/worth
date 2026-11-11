use super::runtime_harness::{attached_consumer, attached_future_consumer, delivery_budget};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

fn continuation_test_identity(label: &str) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "subscription_continuation_test_identity_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("label"), label)
        .seal()
        .expect("fixture digest must fit")
}

#[test]
fn identity_remap_continuation_is_patch_visible_and_changes_window_digest() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::Detail,
        None,
        "consumer-a",
        1,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );
    let window =
        open_query_delivery_window(&mut runtime, &attachment, delivery_budget(2, 2)).unwrap();
    let original_window_digest = window.delivery_window_projection().label().to_string();
    let evidence = admit_subscription_continuation_evidence(
        attachment.lane_digest().clone(),
        SubscriptionContinuationClass::IdentityRemap,
        continuation_test_identity("employee:old"),
        continuation_test_identity("employee:new"),
        continuation_test_identity("basis:current"),
        continuation_test_identity("identity-evolution-authority"),
        ContinuationRemapWidth::measured(1),
    )
    .unwrap();

    let (window, report) =
        apply_active_subscription_continuation(&mut runtime, window, evidence).unwrap();
    let continued_window_digest = window.delivery_window_projection().label().to_string();
    let (delta, continuation_counters) = lower_subscription_continuation_report(&report);
    let (delta, lowering_report, _) = lower_query_subscription_maintenance_delta(delta).unwrap();
    let packet = build_active_delivery_work_packet(
        &mut runtime,
        &attachment,
        delta,
        lowering_report,
        ActiveDeliveryDensityPosture::SparseDelta,
        ActiveDeliveryAffectedLaneWidth::measured(1),
        ActiveDeliveryAffectedAttachmentWidth::measured(1),
        PatchGroupWidth::measured(1),
        ActiveDeliveryContinuationWidth::measured(1),
        ActiveDeliveryPreviewResidueWidth::measured(0),
        ActiveAllocationScopeWidth::measured(1),
        ActiveSubscriptionAllocationPosture::PatchScratch,
    )
    .unwrap();
    let batch = emit_query_delivery_batch(&mut runtime, window, packet).unwrap();

    assert_ne!(continued_window_digest, original_window_digest);
    assert_eq!(report.remap_width(), 1);
    assert_eq!(report.performance_receipt().consumed_width(), 1);
    assert_eq!(report.performance_receipt().budgeted_width(), 1);
    assert_eq!(continuation_counters.continuation_remap_count(), 1);
    assert_eq!(continuation_counters.continuation_remap_width(), 1);
    assert_eq!(
        batch.patch_group().kind(),
        QueryPatchGroupKind::ContinuationPatchGroup
    );
    assert_eq!(batch.counters().continuation_remap_count(), 1);
}

#[test]
fn advisory_and_identity_break_continuations_are_distinct_counters() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::Detail,
        None,
        "consumer-a",
        1,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );

    let advisory = admit_subscription_continuation_evidence(
        attachment.lane_digest().clone(),
        SubscriptionContinuationClass::CorrespondenceAdvisory,
        continuation_test_identity("employee:maybe-old"),
        continuation_test_identity("employee:maybe-new"),
        continuation_test_identity("basis:current"),
        continuation_test_identity("correspondence-authority"),
        ContinuationRemapWidth::measured(1),
    )
    .unwrap();
    let identity_break = admit_subscription_continuation_evidence(
        attachment.lane_digest().clone(),
        SubscriptionContinuationClass::IdentityBreak,
        continuation_test_identity("employee:old"),
        continuation_test_identity("identity-break-terminal"),
        continuation_test_identity("basis:current"),
        continuation_test_identity("identity-evolution-authority"),
        ContinuationRemapWidth::measured(1),
    )
    .unwrap();

    let (_, advisory_report) = apply_subscription_continuation(
        open_query_delivery_window(&mut runtime, &attachment, delivery_budget(2, 2)).unwrap(),
        advisory,
    )
    .unwrap();
    let (_, advisory_counters) = lower_subscription_continuation_report(&advisory_report);
    let (_, break_report) = apply_subscription_continuation(
        open_query_delivery_window(&mut runtime, &attachment, delivery_budget(2, 2)).unwrap(),
        identity_break,
    )
    .unwrap();
    let (_, break_counters) = lower_subscription_continuation_report(&break_report);

    assert_eq!(advisory_counters.continuation_advisory_count(), 1);
    assert_eq!(advisory_counters.continuation_identity_break_count(), 0);
    assert_ne!(
        advisory_report
            .performance_receipt()
            .performance_receipt_projection()
            .label(),
        break_report
            .performance_receipt()
            .performance_receipt_projection()
            .label()
    );
    assert_eq!(break_counters.continuation_identity_break_count(), 1);
    assert_eq!(break_counters.continuation_advisory_count(), 0);
}

#[test]
fn unsupported_continuation_and_foreign_lane_evidence_deny_typed_and_early() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, detail) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::Detail,
        None,
        "consumer-a",
        1,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );
    let (_, collection) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        "consumer-b",
        1,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );

    let unsupported = admit_subscription_continuation_evidence(
        detail.lane_digest().clone(),
        SubscriptionContinuationClass::UnsupportedContinuation,
        continuation_test_identity("employee:old"),
        continuation_test_identity("employee:new"),
        continuation_test_identity("basis:current"),
        continuation_test_identity("identity-evolution-authority"),
        ContinuationRemapWidth::measured(1),
    )
    .unwrap_err();
    assert_eq!(
        unsupported.denial_kind(),
        &SubscriptionContinuationDenialKind::UnsupportedContinuationClass
    );
    assert_eq!(unsupported.counters().continuation_remap_denial_count(), 1);

    let preview_promotion = admit_subscription_continuation_evidence(
        detail.lane_digest().clone(),
        SubscriptionContinuationClass::PreviewPromotionRemap,
        continuation_test_identity("preview:employee:old"),
        continuation_test_identity("authoritative:employee:new"),
        continuation_test_identity("basis:preview"),
        continuation_test_identity("preview-promotion-authority"),
        ContinuationRemapWidth::measured(1),
    )
    .unwrap_err();
    assert_eq!(
        preview_promotion.denial_kind(),
        &SubscriptionContinuationDenialKind::UnsupportedContinuationClass
    );
    assert_eq!(
        preview_promotion
            .counters()
            .continuation_remap_denial_count(),
        1
    );

    let foreign = admit_subscription_continuation_evidence(
        collection.lane_digest().clone(),
        SubscriptionContinuationClass::CollectionMembershipRemap,
        continuation_test_identity("employee:old"),
        continuation_test_identity("employee:new"),
        continuation_test_identity("basis:current"),
        continuation_test_identity("identity-evolution-authority"),
        ContinuationRemapWidth::measured(1),
    )
    .unwrap();
    let window = open_query_delivery_window(&mut runtime, &detail, delivery_budget(2, 2)).unwrap();
    let error = apply_subscription_continuation(window, foreign).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &SubscriptionContinuationDenialKind::ContinuationEvidenceMismatch
    );
    assert_eq!(error.counters().continuation_remap_denial_count(), 1);
}

#[test]
fn future_bearing_continuation_retains_checkpoint_and_future_identity() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let attachment = attached_future_consumer(
        &mut runtime,
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        "consumer-a",
        QuerySubscriptionFutureSelection::temporal_async_with_identity(
            true,
            vec![QuerySubscriptionAsyncRequestIdentityPart::new(
                "request",
                "employees",
            )],
        )
        .expect("fixture digest must fit"),
        1,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );
    let evidence = admit_subscription_continuation_evidence_with_active_identity(
        attachment.lane_digest().clone(),
        SubscriptionContinuationClass::IdentityRemap,
        continuation_test_identity("employee:old"),
        continuation_test_identity("employee:new"),
        attachment.future_selection().clone(),
        attachment.basis_binding_identity().clone(),
        attachment.checkpoint_identity().clone(),
        continuation_test_identity("identity-evolution-authority"),
        ContinuationRemapWidth::measured(1),
    )
    .unwrap();
    let window =
        open_query_delivery_window(&mut runtime, &attachment, delivery_budget(2, 2)).unwrap();

    let (window, report) =
        apply_active_subscription_continuation(&mut runtime, window, evidence).unwrap();

    assert_eq!(
        report
            .future_selection()
            .future_selection_projection()
            .label(),
        attachment
            .future_selection()
            .future_selection_projection()
            .label()
    );
    assert_eq!(
        report.checkpoint_identity(),
        attachment.checkpoint_identity()
    );
    assert_eq!(window.active_lane_digest(), attachment.lane_digest());
}
