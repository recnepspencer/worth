use super::runtime_harness::{activation_for, active_budget, attached_consumer, attachment_budget};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn closing_one_of_multiple_consumers_keeps_shared_lane_alive() {
    let activation = activation_for(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
    );
    let first = admit_active_subscription_lane(
        activation.clone(),
        active_budget(2, 2, ActiveSubscriptionAllocationPosture::LifecycleArena),
    )
    .unwrap();
    let second = admit_active_subscription_lane(
        activation,
        active_budget(2, 2, ActiveSubscriptionAllocationPosture::LifecycleArena),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let first_handle = open_active_subscription_lane(&mut runtime, first).unwrap();
    let second_handle = open_active_subscription_lane(&mut runtime, second).unwrap();
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

    let closeout = close_subscription_lifecycle(
        &mut runtime,
        &first_handle,
        SubscriptionLifecycleCloseRequest::DetachConsumer(first_attachment),
    )
    .unwrap();

    assert_eq!(runtime.lane_count(), 1);
    assert!(!closeout.lane_terminal());
    assert_eq!(
        closeout.closeout_kind(),
        &SubscriptionLifecycleCloseoutKind::ConsumerDetached
    );
    assert_eq!(closeout.counters().consumer_attachment_close_count(), 1);
    assert_eq!(closeout.counters().active_lane_close_count(), 0);

    let final_closeout = close_subscription_lifecycle(
        &mut runtime,
        &second_handle,
        SubscriptionLifecycleCloseRequest::TerminateConsumer(second_attachment),
    )
    .unwrap();
    assert!(final_closeout.lane_terminal());
    assert_eq!(runtime.lane_count(), 0);
    assert_eq!(final_closeout.counters().active_lane_close_count(), 1);
}

#[test]
fn final_consumer_closeout_carries_support_and_performance_receipt() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (handle, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        "consumer-a",
        2,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );
    let expected_future = attachment
        .future_selection()
        .future_selection_projection()
        .label()
        .to_string();
    let expected_basis = attachment.basis_binding_projection().label().to_string();
    let expected_checkpoint = attachment.checkpoint_projection().label().to_string();

    let closeout = close_subscription_lifecycle(
        &mut runtime,
        &handle,
        SubscriptionLifecycleCloseRequest::TerminateConsumer(attachment),
    )
    .unwrap();

    assert!(closeout.lane_terminal());
    assert_eq!(
        closeout.support_profile().runtime_backed_support(),
        &QuerySubscriptionRuntimeBackedSupport::Admitted
    );
    assert_eq!(
        closeout.support_profile().active_lifecycle_support(),
        &QuerySubscriptionActiveLifecycleSupport::Admitted
    );
    assert_eq!(
        closeout.support_profile().lifecycle_closeout_support(),
        &QuerySubscriptionLifecycleCloseoutSupport::Admitted
    );
    assert_eq!(
        closeout.support_profile().durable_support(),
        &QuerySubscriptionDurableSupport::ExplicitDebt
    );
    assert_eq!(
        closeout
            .future_selection()
            .future_selection_projection()
            .label()
            .as_str(),
        expected_future.as_str()
    );
    assert_eq!(
        closeout.basis_binding_projection().label().as_str(),
        expected_basis.as_str()
    );
    assert_eq!(
        closeout.checkpoint_projection().label().as_str(),
        expected_checkpoint.as_str()
    );
    assert_eq!(closeout.performance_receipt().consumed_width(), 2);
    assert_eq!(closeout.performance_receipt().remaining_width(), 0);
    assert!(!closeout.closeout_projection().label().is_empty());
    assert_eq!(runtime.lane_count(), 0);
}

#[test]
fn closing_with_foreign_runtime_handle_denies_typed_and_early() {
    let mut owner_runtime = ActiveSubscriptionRuntime::new();
    let (handle, attachment) = attached_consumer(
        &mut owner_runtime,
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        "consumer-a",
        2,
        2,
        DeliveryBackpressurePolicy::RetainWithinWindow,
    );
    let mut foreign_runtime = ActiveSubscriptionRuntime::new();

    let error = close_subscription_lifecycle(
        &mut foreign_runtime,
        &handle,
        SubscriptionLifecycleCloseRequest::TerminateConsumer(attachment),
    )
    .unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &SubscriptionLifecycleCloseDenialKind::LaneHandleMismatch
    );
    assert_eq!(
        error
            .counters()
            .subscription_lifecycle_closeout_denial_count(),
        1
    );
}

#[test]
fn preview_discard_closeout_can_close_the_runtime_lane() {
    let mut runtime = ActiveSubscriptionRuntime::new();
    let (handle, attachment) = attached_consumer(
        &mut runtime,
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        "preview-a",
        2,
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
        PreviewResidueWidth::measured(0),
    );
    let preview_closeout = discard_preview_subscription(isolation, residue).unwrap();

    let closeout = close_subscription_lifecycle(
        &mut runtime,
        &handle,
        SubscriptionLifecycleCloseRequest::PreviewDiscard(preview_closeout),
    )
    .unwrap();

    assert!(closeout.lane_terminal());
    assert_eq!(
        closeout.closeout_kind(),
        &SubscriptionLifecycleCloseoutKind::PreviewDiscarded
    );
    assert_eq!(
        closeout
            .future_selection()
            .future_selection_projection()
            .label(),
        attachment
            .future_selection()
            .future_selection_projection()
            .label()
    );
    assert_eq!(
        closeout.checkpoint_projection().label(),
        attachment.checkpoint_projection().label()
    );
    assert_eq!(closeout.counters().active_lane_close_count(), 1);
    assert_eq!(closeout.counters().consumer_attachment_close_count(), 1);
    assert_eq!(runtime.lane_count(), 0);
}
