use super::runtime_harness::{
    active_lane_for, active_lane_for_with_context, attached_consumer, attachment_budget,
    zero_authoritative_residue,
};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn preview_discard_zero_authoritative_residue_produces_closeout() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::Detail,
        None,
        "preview-a",
        1,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );
    let isolation = admit_preview_subscription_isolation(
        &attachment,
        "preview-epoch-a",
        PreviewResidueWidth::measured(2),
    )
    .unwrap();
    let residue = zero_authoritative_residue();
    let residue_digest = residue.report_projection().label().to_string();

    let closeout = discard_preview_subscription(isolation, residue).unwrap();

    assert_eq!(closeout.active_lane_digest(), attachment.lane_digest());
    assert_eq!(closeout.attachment_digest(), attachment.attachment_digest());
    assert_eq!(
        closeout.residue_report_projection().label(),
        residue_digest.as_str()
    );
    assert_eq!(closeout.counters().preview_discard_residue_check_count(), 1);
    assert_eq!(closeout.counters().preview_residue_width(), 2);
    assert_eq!(closeout.counters().preview_authoritative_residue_count(), 0);
    assert_eq!(closeout.performance_receipt().consumed_width(), 2);
    assert_eq!(closeout.performance_receipt().budgeted_width(), 2);
}

#[test]
fn preview_discard_with_authoritative_residue_denies_typed_and_early() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::Detail,
        None,
        "preview-a",
        1,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );
    let isolation = admit_preview_subscription_isolation(
        &attachment,
        "preview-epoch-a",
        PreviewResidueWidth::measured(4),
    )
    .unwrap();
    let residue = measure_preview_subscription_residue(
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(1),
    );

    let error = discard_preview_subscription(isolation, residue).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &PreviewSubscriptionIsolationDenialKind::PreviewDiscardResidueDenied
    );
    assert_eq!(error.counters().preview_discard_residue_check_count(), 1);
    assert_eq!(error.counters().preview_authoritative_residue_count(), 2);
    assert_eq!(error.counters().preview_residue_width(), 4);
}

#[test]
fn preview_discard_denies_when_total_residue_exceeds_admitted_budget() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::Detail,
        None,
        "preview-a",
        1,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );
    let isolation = admit_preview_subscription_isolation(
        &attachment,
        "preview-epoch-a",
        PreviewResidueWidth::measured(1),
    )
    .unwrap();
    let residue = measure_preview_subscription_residue(
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(1),
    );

    let error = discard_preview_subscription(isolation, residue).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &PreviewSubscriptionIsolationDenialKind::PreviewDiscardResidueDenied
    );
    assert_eq!(error.counters().preview_discard_residue_check_count(), 1);
    assert_eq!(error.counters().preview_authoritative_residue_count(), 0);
    assert_eq!(error.counters().preview_residue_width(), 2);
}

#[test]
fn preview_authoritative_sharing_is_denied_even_with_matching_lane_digest() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (handle, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::Detail,
        None,
        "preview-a",
        1,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );
    let isolation = admit_preview_subscription_isolation(
        &attachment,
        "preview-epoch-a",
        PreviewResidueWidth::measured(1),
    )
    .unwrap();

    let error = deny_preview_authoritative_sharing(&isolation, &handle).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &PreviewSubscriptionIsolationDenialKind::PreviewAuthoritativeSharingDenied
    );
    assert_eq!(
        error
            .counters()
            .preview_authoritative_sharing_denial_count(),
        1
    );
    assert_eq!(isolation.active_lane_digest(), handle.lane_digest());
}

#[test]
fn preview_promotion_emits_handoff_to_authoritative_lane_without_in_place_mutation() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (_, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::Detail,
        None,
        "preview-a",
        1,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );
    let authoritative_handle = active_lane_for(
        &mut runtime,
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        1,
        2,
    );
    let isolation = admit_preview_subscription_isolation(
        &attachment,
        "preview-epoch-a",
        PreviewResidueWidth::measured(1),
    )
    .unwrap();
    let isolation_digest = isolation.isolation_projection().label().to_string();
    let residue_report = measure_preview_subscription_residue(
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(0),
    );

    let handoff = promote_preview_subscription(
        isolation,
        &residue_report,
        &authoritative_handle,
        "promotion-authority",
    )
    .unwrap();

    assert_eq!(
        handoff.authoritative_active_lane_digest(),
        authoritative_handle.lane_digest()
    );
    assert_ne!(
        handoff.preview_lane_digest(),
        handoff.authoritative_active_lane_digest()
    );
    assert_ne!(
        handoff.preview_basis_binding_projection().label(),
        handoff.authoritative_basis_binding_projection().label()
    );
    assert_ne!(
        handoff.handoff_projection().label(),
        isolation_digest.as_str()
    );
    assert_eq!(
        handoff.residue_report_projection().label(),
        residue_report.report_projection().label()
    );
    assert!(!handoff.rebinding_projection().label().is_empty());
    assert_eq!(handoff.counters().preview_promotion_handoff_count(), 1);
    assert_eq!(handoff.performance_receipt().consumed_width(), 1);
    assert_eq!(handoff.performance_receipt().remaining_width(), 0);
}

#[test]
fn preview_residue_classes_make_authoritative_residue_explicit() {
    let report = measure_preview_subscription_residue(
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(2),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(1),
        PreviewResidueWidth::measured(0),
        PreviewResidueWidth::measured(3),
        PreviewResidueWidth::measured(4),
    );

    assert!(PreviewSubscriptionResidueClass::AuthoritativeRouting.is_authoritative());
    assert!(!PreviewSubscriptionResidueClass::TemporaryPreviewExecution.is_authoritative());
    assert_eq!(
        report.class_width(PreviewSubscriptionResidueClass::AuthoritativeCheckpoint),
        2
    );
    assert_eq!(report.authoritative_residue_width(), 4);
    assert_eq!(report.temporary_residue_width(), 7);
    assert_eq!(report.preview_residue_width(), 11);
}

#[test]
fn future_preview_isolation_retains_basis_and_checkpoint_identity() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let preview_handle = active_lane_for_with_context(
        &mut runtime,
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionBasisPosture::PreviewScoped,
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
    );
    let preview_attachment = attach_subscription_consumer(
        &mut runtime,
        &preview_handle,
        SubscriptionConsumerAttachmentRequest::admitted("preview-a", "cursor")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow),
    )
    .unwrap();
    let authoritative_handle = active_lane_for_with_context(
        &mut runtime,
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionBasisPosture::CurrentHead,
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
    );
    let isolation = admit_preview_subscription_isolation(
        &preview_attachment,
        "preview-epoch-future",
        PreviewResidueWidth::measured(1),
    )
    .unwrap();

    let error = deny_preview_authoritative_sharing(&isolation, &authoritative_handle).unwrap_err();

    assert_eq!(
        isolation
            .future_selection()
            .future_selection_projection()
            .label(),
        preview_attachment
            .future_selection()
            .future_selection_projection()
            .label()
    );
    assert_eq!(
        isolation.checkpoint_projection().label(),
        preview_attachment.checkpoint_projection().label()
    );
    assert_ne!(
        isolation.basis_binding_projection().label(),
        authoritative_handle.basis_binding_projection().label()
    );
    assert_ne!(
        isolation.checkpoint_projection().label(),
        authoritative_handle.checkpoint_projection().label()
    );
    assert_eq!(
        error.denial_kind(),
        &PreviewSubscriptionIsolationDenialKind::PreviewAuthoritativeSharingDenied
    );
}
