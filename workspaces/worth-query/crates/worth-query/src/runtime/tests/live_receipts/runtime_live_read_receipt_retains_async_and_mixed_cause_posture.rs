//! runtime live read receipt retains async and mixed cause posture precedence.
use super::*;

#[test]
fn runtime_live_read_receipt_retains_async_and_mixed_cause_posture_precedence() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut async_runtime = stateful_bridge_task_runtime(request_execution);
    let async_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = async_runtime
        .declare_live_view(
            request_execution,
            "external.tasks.async",
            task_live_request(),
            task_schema(),
        )
        .expect("async live view should declare");
    let (basis_digest, generation_digest) =
        live_subscription_async_identity(&async_runtime, async_view.name());
    async_runtime
        .project_async_result_state(
            async_view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                "async:live-receipt-current",
            ),
            &basis_digest,
            &generation_digest,
        )
        .expect("async result state should project");
    let async_receipt = async_runtime
        .read_live_result(request_execution, &async_view)
        .expect("async live read should execute")
        .receipt()
        .clone();
    let async_posture = async_receipt
        .materialized_fact_posture()
        .expect("async live read receipt should retain posture");

    assert_eq!(
        async_posture.kind(),
        ProjectionMaterializedFactPostureKind::AsyncBacked
    );
    assert_eq!(
        async_posture.basis_digest(),
        async_receipt.snapshot_evidence_identity().as_str()
    );

    let bridge = test_bridge();
    let truth_patch = canonical_truth_patch("truth-main", "snapshot-a", "commit-a", "patch-a");
    let truth_plus_time =
        authoritative_truth_plus_time_cause(&bridge, &truth_patch, resource_request);
    let async_completion = admitted_async_completion(
        &bridge,
        worth_signal::facade::NodeId::new(243, 0),
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
        .expect("mixed-cause delivery window should plan");
    let mut mixed_runtime = stateful_bridge_task_runtime(request_execution);
    let mixed_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = mixed_runtime
        .declare_live_view(
            request_execution,
            "external.tasks.mixed",
            task_live_request(),
            task_schema(),
        )
        .expect("mixed live view should declare");
    mixed_runtime
        .emit_mixed_cause_delivery(mixed_view.name(), &ordering, &window)
        .expect("mixed-cause delivery should emit");
    let mixed_receipt = mixed_runtime
        .read_live_result(request_execution, &mixed_view)
        .expect("mixed-cause live read should execute")
        .receipt()
        .clone();
    let mixed_posture = mixed_receipt
        .materialized_fact_posture()
        .expect("mixed-cause live read receipt should retain posture");

    assert_eq!(
        mixed_posture.kind(),
        ProjectionMaterializedFactPostureKind::MixedCause
    );
    assert_eq!(
        mixed_posture.basis_digest(),
        mixed_receipt.snapshot_evidence_identity().as_str()
    );
    assert_eq!(
        mixed_posture.runtime_origin_digest(),
        Some(
            mixed_view
                .subscription_installation()
                .installation_projection()
                .label()
                .as_str()
        )
    );
}
