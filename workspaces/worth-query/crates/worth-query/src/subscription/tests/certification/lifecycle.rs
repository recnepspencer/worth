use super::lifecycle_world::active_lifecycle_certification_for;
use super::preview_certification_world::preview_certification;
use super::*;
use crate::live::LiveQueryFamily;
use crate::subscription::evidence_identities::lifecycle_absent_preview_isolation_identity;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn lifecycle_certification_emits_runtime_backed_bundle() {
    let artifacts = active_lifecycle_certification_for(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        1,
        0,
    );

    let bundle = certify_subscription_lifecycle(
        artifacts.context,
        &artifacts.admission,
        &artifacts.activation,
        &artifacts.scale_report,
        &artifacts.active_admission,
        &artifacts.handle,
        &artifacts.attachment,
        artifacts.delivery_batch.delivery_window_identity(),
        &artifacts.delta,
        &artifacts.lowering_report,
        &artifacts.work_packet,
        &artifacts.delivery_batch,
        &artifacts.acknowledged_attachment,
        artifacts.continuation_report.as_ref(),
        preview_certification(&artifacts.preview),
        &artifacts.closeout,
    )
    .unwrap();

    assert!(!bundle.certification_bundle_projection().label().is_empty());
    assert_eq!(
        bundle.active_lane_projection().label(),
        artifacts.handle.lane_projection().label()
    );
    assert_eq!(
        bundle.delivery_receipt_projection().label(),
        artifacts
            .delivery_batch
            .receipt()
            .receipt_projection()
            .label()
    );
    assert_eq!(
        bundle.acknowledgement_frontier_projection().label(),
        artifacts
            .acknowledged_attachment
            .acknowledgement_frontier()
            .frontier_projection()
            .label()
    );
    assert!(artifacts.continuation_report.is_none());
    assert_eq!(
        bundle.preview_isolation_projection().label(),
        lifecycle_absent_preview_isolation_identity().as_str()
    );
    assert!(!bundle.support_matrix_projection().label().is_empty());
    assert!(!bundle.counter_snapshot_projection().label().is_empty());
    assert!(
        !bundle.counter_sequence_identity().as_str().is_empty(),
        "lifecycle certification should bind typed counter sequence identity"
    );
    assert!(
        !bundle.certification_bundle_identity().as_str().is_empty(),
        "certification bundle authority must be typed evidence identity"
    );
}

#[test]
fn lifecycle_certification_binds_continuation_receipt_and_digest() {
    let artifacts = active_lifecycle_certification_for(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        1,
        1,
    );

    let bundle = certify_subscription_lifecycle(
        artifacts.context,
        &artifacts.admission,
        &artifacts.activation,
        &artifacts.scale_report,
        &artifacts.active_admission,
        &artifacts.handle,
        &artifacts.attachment,
        artifacts.delivery_batch.delivery_window_identity(),
        &artifacts.delta,
        &artifacts.lowering_report,
        &artifacts.work_packet,
        &artifacts.delivery_batch,
        &artifacts.acknowledged_attachment,
        artifacts.continuation_report.as_ref(),
        preview_certification(&artifacts.preview),
        &artifacts.closeout,
    )
    .unwrap();

    assert!(artifacts.continuation_report.is_some());
    assert_ne!(bundle.continuation_projection().label(), "none");
    assert!(!bundle
        .subscription_performance_receipt_projection()
        .label()
        .is_empty());
}

#[test]
fn lifecycle_certification_denies_attachment_from_foreign_lane() {
    let control = active_lifecycle_certification_for(
        LiveQueryFamily::Detail,
        None,
        QuerySubscriptionMaintenanceDeltaKind::DetailFieldDelta,
        1,
        0,
    );
    let foreign = active_lifecycle_certification_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionMaintenanceDeltaKind::CollectionMembershipDelta,
        1,
        0,
    );

    let error = certify_subscription_lifecycle(
        control.context,
        &control.admission,
        &control.activation,
        &control.scale_report,
        &control.active_admission,
        &control.handle,
        &foreign.attachment,
        control.delivery_batch.delivery_window_identity(),
        &control.delta,
        &control.lowering_report,
        &control.work_packet,
        &control.delivery_batch,
        &control.acknowledged_attachment,
        control.continuation_report.as_ref(),
        preview_certification(&control.preview),
        &control.closeout,
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &SubscriptionLifecycleCertificationDenialKind::AttachmentSourceMismatch
    );
    assert!(!error
        .failure_projection()
        .expect("fixture digest must fit")
        .label()
        .is_empty());
}
