use super::runtime_harness::{activation_for_future, active_budget, attachment_budget};
use super::*;
use crate::live::LiveQueryFamily;
use crate::view_shape_live::LiveViewShapeFamily;

#[test]
fn temporal_async_and_mixed_live_meaning_become_real_active_lanes() {
    let scenarios = [
        (
            QuerySubscriptionFutureSelection::temporal(),
            QuerySubscriptionFutureSelectionClass::Temporal,
        ),
        (
            QuerySubscriptionFutureSelection::async_resource_with_identity(
                true,
                vec![QuerySubscriptionAsyncRequestIdentityPart::new(
                    "request",
                    "employees",
                )],
            ),
            QuerySubscriptionFutureSelectionClass::AsyncResource,
        ),
        (
            QuerySubscriptionFutureSelection::temporal_async_with_identity(
                true,
                vec![QuerySubscriptionAsyncRequestIdentityPart::new(
                    "request",
                    "employees",
                )],
            ),
            QuerySubscriptionFutureSelectionClass::TemporalAsync,
        ),
    ];

    for (future_selection, expected_class) in scenarios {
        let activation = activation_for_future(
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            future_selection.expect("fixture digest must fit"),
        );
        let future_digest = activation
            .future_selection()
            .future_selection_projection()
            .label()
            .to_string();
        let checkpoint_digest = activation.checkpoint_projection().label().to_string();
        let admission = admit_active_subscription_lane(
            activation,
            active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
        )
        .unwrap();
        let mut runtime = ActiveSubscriptionRuntime::new();
        let handle = open_active_subscription_lane(&mut runtime, admission.clone()).unwrap();

        assert_eq!(admission.future_selection().class(), expected_class);
        assert_eq!(
            admission
                .future_selection()
                .future_selection_projection()
                .label()
                .as_str(),
            future_digest.as_str()
        );
        assert_eq!(
            admission.checkpoint_projection().label().as_str(),
            checkpoint_digest.as_str()
        );
        assert_eq!(handle.future_selection().class(), expected_class);
        assert_eq!(
            handle.checkpoint_projection().label().as_str(),
            checkpoint_digest.as_str()
        );
        assert_eq!(runtime.lane_count(), 1);
    }
}

#[test]
fn future_equivalent_consumers_share_one_lane_and_retain_lane_owned_future_identity() {
    let activation = activation_for_future(
        LiveQueryFamily::OrderedCollection,
        Some(LiveViewShapeFamily::Table),
        QuerySubscriptionFutureSelection::temporal_async_with_identity(
            true,
            vec![QuerySubscriptionAsyncRequestIdentityPart::new(
                "request",
                "employees",
            )],
        )
        .expect("fixture digest must fit"),
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
    let expected_checkpoint = first.checkpoint_projection().label().to_string();
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
        first_handle.future_selection(),
        second_handle.future_selection()
    );
    assert_eq!(
        first_handle.checkpoint_projection().label().as_str(),
        expected_checkpoint.as_str()
    );
    assert_eq!(
        first_attachment.future_selection(),
        second_attachment.future_selection()
    );
    assert_eq!(
        first_attachment.checkpoint_projection().label(),
        second_attachment.checkpoint_projection().label()
    );
}

#[test]
fn async_request_identity_mismatch_denies_semantic_lane_join() {
    let first = admit_active_subscription_lane(
        activation_for_future(
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            QuerySubscriptionFutureSelection::async_resource_with_identity(
                true,
                vec![QuerySubscriptionAsyncRequestIdentityPart::new(
                    "request",
                    "employees-a",
                )],
            )
            .expect("fixture digest must fit"),
        ),
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mismatch = admit_active_subscription_lane(
        activation_for_future(
            LiveQueryFamily::OrderedCollection,
            Some(LiveViewShapeFamily::Table),
            QuerySubscriptionFutureSelection::async_resource_with_identity(
                true,
                vec![QuerySubscriptionAsyncRequestIdentityPart::new(
                    "request",
                    "employees-b",
                )],
            )
            .expect("fixture digest must fit"),
        ),
        active_budget(1, 2, ActiveSubscriptionAllocationPolicy::LifecycleArena),
    )
    .unwrap();
    let mut runtime = ActiveSubscriptionRuntime::new();
    let handle = open_active_subscription_lane(&mut runtime, first).unwrap();

    let error = join_active_subscription_lane(&mut runtime, &handle, mismatch).unwrap_err();

    assert_eq!(
        error.denial_kind(),
        &ActiveSubscriptionLifecycleDenialKind::RegistryEquivalenceMismatch
    );
    assert_eq!(error.counters().active_lane_join_denial_count(), 1);
}
