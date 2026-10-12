use super::live::{closed, open_task_resource, task_workspace};
use crate::ordinary::live::WorthQueryManagedLiveDeliveryCauseKind;
use crate::runtime::tests::support::*;
use worth_runtime_bridge::facade::{
    BridgeAsyncRequestTruthViewBasis, BridgeMixedCauseOrderingInput,
    BridgeMixedCauseOrderingLaneKind, BridgeMixedCauseOrderingRequest,
    BridgeSubscriptionDeliveryFamilyKind,
};

#[test]
fn managed_delivery_keeps_async_and_temporal_cause_families_distinct() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let bridge = test_bridge();
    let async_completion = admitted_async_completion(
        &bridge,
        worth_signal::facade::NodeId::new(601, 0),
        BridgeAsyncRequestTruthViewBasis::authoritative(
            TruthBranchIdentity::from_bridge_harness_label("truth-main"),
            TruthCommitIdentity::from_bridge_harness_label("commit-a"),
            TruthSnapshotIdentity::from_bridge_harness_label("snapshot-a"),
        ),
        64,
    );
    let ordering = bridge.order_mixed_causes(&BridgeMixedCauseOrderingRequest::new(
        BridgeMixedCauseOrderingLaneKind::Authoritative,
        vec![BridgeMixedCauseOrderingInput::AsyncCompletion(
            async_completion,
        )],
    ));
    let window = bridge
        .plan_mixed_cause_delivery_window(
            &ordering,
            BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
        )
        .expect("async delivery window should plan");
    let mut workspace = task_workspace(request_execution, "managed-live-distinct-delivery-causes");
    let handle = open_task_resource(
        request_execution,
        &mut workspace,
        "tasks.distinct-delivery-causes",
    );

    workspace
        .runtime
        .emit_mixed_cause_delivery(handle.name(), &ordering, &window)
        .expect("async delivery should emit through the managed resource");
    let async_delivery = handle
        .drain(&mut workspace)
        .expect("async delivery should drain through the ordinary handle");
    assert_eq!(async_delivery.batches().len(), 1);
    assert_eq!(
        async_delivery.batches()[0].cause_kind(),
        WorthQueryManagedLiveDeliveryCauseKind::AsyncCompletion
    );
    assert!(async_delivery.batches()[0].maintenance_work().is_none());

    workspace
        .runtime
        .emit_time_only_delivery(
            handle.name(),
            crate::subscription::QuerySubscriptionDeliveryCauseKind::Deadline,
            "tick:managed-deadline",
            false,
            true,
        )
        .expect("temporal delivery should emit through the managed resource");
    let temporal_delivery = handle
        .drain(&mut workspace)
        .expect("temporal delivery should drain through the ordinary handle");
    assert_eq!(temporal_delivery.batches().len(), 1);
    assert_eq!(
        temporal_delivery.batches()[0].cause_kind(),
        WorthQueryManagedLiveDeliveryCauseKind::Temporal
    );
    assert_ne!(
        async_delivery.batches()[0].cause_identity(),
        temporal_delivery.batches()[0].cause_identity()
    );
    assert!(closed(handle.close(&mut workspace)).lane_terminal());
}
