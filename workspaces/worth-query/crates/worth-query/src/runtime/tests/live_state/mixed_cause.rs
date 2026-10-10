use super::super::support::*;
use crate::subscription::QuerySubscriptionDeliveryCauseKind;
use worth_runtime_bridge::facade::{
    BridgeAsyncRequestTruthViewBasis, BridgeMixedCauseOrderingInput,
    BridgeMixedCauseOrderingLaneKind, BridgeMixedCauseOrderingRequest,
    BridgeSubscriptionDeliveryFamilyKind,
};

#[test]
fn runtime_mixed_cause_delivery_replays_canonically_across_shuffled_bridge_input_order() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bridge = test_bridge();
    let truth_patch = canonical_truth_patch("truth-main", "snapshot-a", "commit-a", "patch-a");
    let truth_plus_time =
        authoritative_truth_plus_time_cause(&bridge, &truth_patch, resource_request);
    let time_only = authoritative_time_only_cause(&bridge, resource_request);
    let async_completion = admitted_async_completion(
        &bridge,
        worth_signal::facade::NodeId::new(241, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            TruthBranchIdentity::from_bridge_harness_label("truth-main"),
            TruthCommitIdentity::from_bridge_harness_label("commit-a"),
            TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
        ),
        64,
    );

    let first = bridge.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::AsyncCompletion(async_completion.clone()),
            BridgeMixedCauseOrderingInput::Temporal(time_only.clone()),
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch.clone()),
            BridgeMixedCauseOrderingInput::Temporal(truth_plus_time.clone()),
        ],
    ));
    let second = bridge.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::Temporal(truth_plus_time),
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch),
            BridgeMixedCauseOrderingInput::AsyncCompletion(async_completion),
            BridgeMixedCauseOrderingInput::Temporal(time_only),
        ],
    ));
    let first_window = bridge
        .plan_mixed_cause_delivery_window(
            &first,
            BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
        )
        .expect("mixed-cause delivery window should plan");
    let second_window = bridge
        .plan_mixed_cause_delivery_window(
            &second,
            BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
        )
        .expect("mixed-cause delivery window should plan");

    let mut runtime_a = stateful_bridge_task_runtime();
    let view_a: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime_a
        .declare_live_view("tasks.mixed-cause-a", task_live_request(), task_schema())
        .expect("first live view should declare");
    let mut runtime_b = stateful_bridge_task_runtime();
    let view_b: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime_b
        .declare_live_view("tasks.mixed-cause-b", task_live_request(), task_schema())
        .expect("second live view should declare");

    runtime_a
        .emit_mixed_cause_delivery(view_a.name(), &first, &first_window)
        .expect("first mixed-cause delivery should emit");
    runtime_b
        .emit_mixed_cause_delivery(view_b.name(), &second, &second_window)
        .expect("second mixed-cause delivery should emit");

    let batch_a = runtime_a
        .drain_patches(&view_a)
        .query_delivery_batches
        .remove(0);
    let batch_b = runtime_b
        .drain_patches(&view_b)
        .query_delivery_batches
        .remove(0);

    assert_eq!(
        batch_a.mixed_cause_delivery().ordering_for_reporting(),
        batch_b.mixed_cause_delivery().ordering_for_reporting()
    );
    assert_eq!(
        batch_a.mixed_cause_delivery().mixed_cause_for_reporting(),
        batch_b.mixed_cause_delivery().mixed_cause_for_reporting()
    );
    assert_eq!(
        batch_a.mixed_cause_delivery().ordered_member_kinds(),
        batch_b.mixed_cause_delivery().ordered_member_kinds()
    );
    assert_eq!(
        batch_a.mixed_cause_delivery().coalescing_kind(),
        WorthQueryRuntimeDeliveryCoalescingKind::Coalesced
    );
    assert_eq!(
        batch_a.delivery_cause_kind(),
        QuerySubscriptionDeliveryCauseKind::MixedCause
    );
    assert!(batch_a.has_relational_patch());
    assert_eq!(
        batch_a.mixed_cause_delivery().ordered_member_kinds(),
        &[
            WorthQueryRuntimeMixedCauseMemberKind::TruthPatch,
            WorthQueryRuntimeMixedCauseMemberKind::TemporalTruthPlusTime,
            WorthQueryRuntimeMixedCauseMemberKind::AsyncCompletion,
            WorthQueryRuntimeMixedCauseMemberKind::TemporalTimeOnly,
        ]
    );
}

#[test]
fn runtime_mixed_cause_delivery_retains_duplicate_suppression_explicitly() {
    let bridge = test_bridge();
    let truth_patch = canonical_truth_patch("truth-main", "snapshot-a", "commit-a", "patch-a");
    let ordering = bridge.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch.clone()),
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch),
        ],
    ));
    let window = bridge
        .plan_mixed_cause_delivery_window(
            &ordering,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        )
        .expect("single ordered cause should still plan a delivery window");

    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("tasks.mixed-duplicate", task_live_request(), task_schema())
        .expect("live view should declare");
    runtime
        .emit_mixed_cause_delivery(view.name(), &ordering, &window)
        .expect("duplicate-suppressed mixed-cause delivery should emit");
    let batch = runtime
        .drain_patches(&view)
        .query_delivery_batches
        .remove(0);

    assert_eq!(
        batch.delivery_cause_kind(),
        QuerySubscriptionDeliveryCauseKind::RelationalPatch
    );
    assert_eq!(
        batch.mixed_cause_delivery().coalescing_kind(),
        WorthQueryRuntimeDeliveryCoalescingKind::Atomic
    );
    assert_eq!(batch.mixed_cause_delivery().ordered_member_kinds().len(), 1);
    assert_eq!(
        batch
            .mixed_cause_delivery()
            .suppressed_cause_identities()
            .len(),
        1
    );
    assert!(batch
        .mixed_cause_delivery()
        .denied_cause_identities()
        .is_empty());
}

#[test]
fn runtime_mixed_cause_delivery_preserves_denied_preview_boundary_without_coalescing_it() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bridge = test_bridge();
    let truth_patch = canonical_truth_patch("truth-main", "snapshot-a", "commit-a", "patch-a");
    let preview_cause = preview_time_only_cause(&bridge, "mixed-preview", resource_request);
    let ordering = bridge.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch),
            BridgeMixedCauseOrderingInput::Temporal(preview_cause),
        ],
    ));
    let window = bridge
        .plan_mixed_cause_delivery_window(
            &ordering,
            BridgeSubscriptionDeliveryFamilyKind::CanonicalMember,
        )
        .expect("authoritative truth patch should still admit a delivery window");

    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            "tasks.mixed-preview-boundary",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    runtime
        .emit_mixed_cause_delivery(view.name(), &ordering, &window)
        .expect("authoritative mixed-cause delivery should emit");
    let batch = runtime
        .drain_patches(&view)
        .query_delivery_batches
        .remove(0);

    assert_eq!(
        batch.delivery_cause_kind(),
        QuerySubscriptionDeliveryCauseKind::RelationalPatch
    );
    assert_eq!(
        batch.mixed_cause_delivery().ordered_member_kinds(),
        &[WorthQueryRuntimeMixedCauseMemberKind::TruthPatch]
    );
    assert!(batch
        .mixed_cause_delivery()
        .suppressed_cause_identities()
        .is_empty());
    assert_eq!(
        batch.mixed_cause_delivery().denied_cause_identities().len(),
        1
    );
    assert!(batch.has_relational_patch());
}

#[test]
fn runtime_state_and_inspection_retain_mixed_cause_delivery_projection_after_drain() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let bridge = test_bridge();
    let truth_patch = canonical_truth_patch("truth-main", "snapshot-a", "commit-a", "patch-a");
    let truth_plus_time =
        authoritative_truth_plus_time_cause(&bridge, &truth_patch, resource_request);
    let async_completion = admitted_async_completion(
        &bridge,
        worth_signal::facade::NodeId::new(242, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            TruthBranchIdentity::from_bridge_harness_label("truth-main"),
            TruthCommitIdentity::from_bridge_harness_label("commit-a"),
            TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
        ),
        64,
    );
    let ordering = bridge.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![
            BridgeMixedCauseOrderingInput::AsyncCompletion(async_completion),
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch),
            BridgeMixedCauseOrderingInput::Temporal(truth_plus_time),
        ],
    ));
    let window = bridge
        .plan_mixed_cause_delivery_window(
            &ordering,
            BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
        )
        .expect("coalesced mixed-cause delivery window should plan");

    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            "tasks.mixed-state-inspection",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    runtime
        .emit_mixed_cause_delivery(view.name(), &ordering, &window)
        .expect("mixed-cause delivery should emit");
    let _drained = runtime.drain_patches(&view);

    let snapshot =
        <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(
            &view, &runtime,
        )
        .expect("state snapshot should remain available after drain");
    assert!(snapshot
        .explanation()
        .contains("mixed-cause delivery is `coalesced`"));
    assert!(snapshot
        .explanation()
        .contains("truth_patch,temporal_truth_plus_time,async_completion"));

    let inspection = runtime
        .inspect(&view)
        .expect("inspection should remain available after drain");
    let WorthQueryInspection::LiveView(inspection) = inspection else {
        panic!("inspection should target the live-view surface");
    };
    let mixed_cause_delivery = inspection
        .mixed_cause_delivery()
        .expect("inspection should retain mixed-cause delivery artifact");
    assert_eq!(
        mixed_cause_delivery.coalescing_kind(),
        WorthQueryRuntimeDeliveryCoalescingKind::Coalesced
    );
    assert_eq!(
        mixed_cause_delivery.ordered_member_kinds(),
        &[
            WorthQueryRuntimeMixedCauseMemberKind::TruthPatch,
            WorthQueryRuntimeMixedCauseMemberKind::TemporalTruthPlusTime,
            WorthQueryRuntimeMixedCauseMemberKind::AsyncCompletion,
        ]
    );
}
