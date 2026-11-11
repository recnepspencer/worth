use super::runtime_harness::{activation_for, active_budget, attachment_budget, emitted_receipt};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn equivalent_consumers_share_one_lane_but_keep_distinct_attachment_state() {
    let activation = activation_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let first = admit_active_subscription_lane(
        activation.clone(),
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let duplicate = admit_active_subscription_lane(
        activation,
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let first_handle = open_active_subscription_lane(&mut runtime, first).unwrap();
    let second_handle = open_active_subscription_lane(&mut runtime, duplicate).unwrap();

    let first_attachment = attach_subscription_consumer(
        &mut runtime,
        &first_handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-a", "cursor-a")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow),
    )
    .unwrap();
    let second_attachment = attach_subscription_consumer(
        &mut runtime,
        &second_handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-b", "cursor-b")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow),
    )
    .unwrap();

    assert_eq!(runtime.lane_count(), 1);
    assert_eq!(
        first_attachment.lane_digest(),
        second_attachment.lane_digest()
    );
    assert_ne!(
        first_attachment.attachment_digest(),
        second_attachment.attachment_digest()
    );
    assert_ne!(
        first_attachment.delivery_cursor_projection().label(),
        second_attachment.delivery_cursor_projection().label()
    );
    assert_eq!(second_attachment.fanout_report().shared_lane_count(), 1);
    assert_eq!(second_attachment.fanout_report().fanout_width(), 2);
    assert_eq!(first_attachment.performance_receipt().consumed_width(), 3);
    assert_eq!(second_attachment.performance_receipt().budgeted_width(), 4);
    assert_eq!(runtime.counters().consumer_attachment_count(), 1);
    assert_eq!(runtime.counters().affected_consumer_attachment_width(), 2);
}

#[test]
fn stale_or_foreign_lane_handle_denies_consumer_attachment() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let admission = admit_active_subscription_lane(
        activation,
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mut owner_runtime = ActiveSubscriptionRuntime::new();
    let handle = open_active_subscription_lane(&mut owner_runtime, admission).unwrap();
    let mut foreign_runtime = ActiveSubscriptionRuntime::new();

    let error = attach_subscription_consumer(
        &mut foreign_runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-a", "cursor-a")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &SubscriptionConsumerAttachmentDenialKind::LaneHandleMismatch
    );
    assert_eq!(error.counters().consumer_attachment_denial_count(), 1);
}

#[test]
fn acknowledgement_requires_receipt_for_same_attachment_and_advancing_sequence() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let admission = admit_active_subscription_lane(
        activation,
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let handle = open_active_subscription_lane(&mut runtime, admission).unwrap();
    let attachment = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-a", "cursor-a")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow),
    )
    .unwrap();
    let receipt = emitted_receipt(&mut runtime, &attachment, "employee.name", 1, 1);
    let stale_receipt = receipt.clone();

    let attachment =
        advance_subscription_acknowledgement(&mut runtime, attachment, receipt).unwrap();

    assert_eq!(
        attachment
            .acknowledgement_frontier()
            .acknowledged_sequence()
            .get(),
        1
    );
    assert_eq!(
        runtime.counters().acknowledgement_frontier_advance_count(),
        1
    );

    let error =
        advance_subscription_acknowledgement(&mut runtime, attachment, stale_receipt).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &SubscriptionConsumerAttachmentDenialKind::AcknowledgementSequenceRegression
    );
    assert_eq!(
        error
            .counters()
            .acknowledgement_sequence_regression_denial_count(),
        1
    );
}

#[test]
fn acknowledgement_receipt_from_another_consumer_is_denied() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let first = admit_active_subscription_lane(
        activation.clone(),
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let duplicate = admit_active_subscription_lane(
        activation,
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let first_handle = open_active_subscription_lane(&mut runtime, first).unwrap();
    let second_handle = open_active_subscription_lane(&mut runtime, duplicate).unwrap();
    let first_attachment = attach_subscription_consumer(
        &mut runtime,
        &first_handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-a", "cursor-a")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow),
    )
    .unwrap();
    let second_attachment = attach_subscription_consumer(
        &mut runtime,
        &second_handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-b", "cursor-b")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow),
    )
    .unwrap();
    let wrong_receipt = emitted_receipt(&mut runtime, &second_attachment, "employee.name", 1, 1);

    let error = advance_subscription_acknowledgement(&mut runtime, first_attachment, wrong_receipt)
        .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &SubscriptionConsumerAttachmentDenialKind::AcknowledgementReceiptMismatch
    );
    assert_eq!(
        error
            .counters()
            .acknowledgement_receipt_mismatch_denial_count(),
        1
    );
}

#[test]
fn slow_consumer_gap_policy_changes_only_that_consumer_delivery_digest() {
    let activation = activation_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let first = admit_active_subscription_lane(
        activation.clone(),
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let duplicate = admit_active_subscription_lane(
        activation,
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let first_handle = open_active_subscription_lane(&mut runtime, first).unwrap();
    let second_handle = open_active_subscription_lane(&mut runtime, duplicate).unwrap();
    let normal = attach_subscription_consumer(
        &mut runtime,
        &first_handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-a", "cursor-a")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow),
    )
    .unwrap();
    let slow = attach_subscription_consumer(
        &mut runtime,
        &second_handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-b", "cursor-b")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::DropWithGapNotice),
    )
    .unwrap();

    assert_eq!(normal.lane_digest(), slow.lane_digest());
    assert_ne!(
        normal.delivery_cursor_projection().label(),
        slow.delivery_cursor_projection().label()
    );
    assert_eq!(
        slow.backpressure_policy(),
        &DeliveryBackpressurePolicy::DropWithGapNotice
    );
    assert_ne!(
        normal
            .performance_receipt()
            .performance_receipt_projection()
            .label(),
        slow.performance_receipt()
            .performance_receipt_projection()
            .label()
    );
    assert_eq!(runtime.counters().delivery_gap_notice_count(), 1);
}

#[test]
fn shared_consumers_emit_independent_fanout_delivery_evidence() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let first = admit_active_subscription_lane(
        activation.clone(),
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let duplicate = admit_active_subscription_lane(
        activation,
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let first_handle = open_active_subscription_lane(&mut runtime, first).unwrap();
    let second_handle = open_active_subscription_lane(&mut runtime, duplicate).unwrap();
    let first_attachment = attach_subscription_consumer(
        &mut runtime,
        &first_handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-a", "cursor-a")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow),
    )
    .unwrap();
    let second_attachment = attach_subscription_consumer(
        &mut runtime,
        &second_handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-b", "cursor-b")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow),
    )
    .unwrap();
    let first_receipt = emitted_receipt(&mut runtime, &first_attachment, "employee.name", 1, 1);
    let first_fanout_delivery = runtime.counters().fanout_delivery_count();
    let second_receipt = emitted_receipt(&mut runtime, &second_attachment, "employee.name", 1, 1);
    let second_fanout_delivery = runtime.counters().fanout_delivery_count();

    assert_eq!(first_receipt.sequence().get(), 1);
    assert_eq!(second_receipt.sequence().get(), 1);
    assert_eq!(first_fanout_delivery, 1);
    assert_eq!(second_fanout_delivery, 1);
    assert_eq!(runtime.lane_count(), 1);
}

#[test]
fn inadmissible_backpressure_request_denies_before_attachment_exists() {
    let activation = activation_for(LiveQueryFamily::Detail, None);
    let admission = admit_active_subscription_lane(
        activation,
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let handle = open_active_subscription_lane(&mut runtime, admission).unwrap();

    let error = attach_subscription_consumer(
        &mut runtime,
        &handle,
        SubscriptionConsumerAttachmentRequest::admitted("consumer-a", "cursor-a")
            .expect("fixture digest must fit"),
        attachment_budget(2, DeliveryBackpressurePolicy::RetainWithinWindow)
            .with_backpressure_denial_request(),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &SubscriptionConsumerAttachmentDenialKind::BackpressureDenied
    );
    assert_eq!(error.counters().backpressure_denial_count(), 1);
}
