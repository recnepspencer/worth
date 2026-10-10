use super::super::support::*;
use crate::ordinary_outcome::{
    WorthQueryOrdinaryRuntimeAsyncPostureKind, WorthQueryOrdinaryRuntimeBasisPostureKind,
    WorthQueryOrdinaryRuntimeCausePostureKind, WorthQueryOrdinaryRuntimePosture,
    WorthQueryOrdinaryRuntimePostureKind,
};
use crate::runtime::async_result_state::runtime_async_checkpoint_label_identity;
use crate::subscription::QuerySubscriptionDeliveryCauseKind;
use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionClass, BridgeAsyncCompletionDenialClass, BridgeAsyncCompletionState,
    BridgeAsyncRequestTruthViewBasis, BridgeMixedCauseOrderingInput,
    BridgeMixedCauseOrderingLaneKind, BridgeMixedCauseOrderingRequest,
    BridgeSubscriptionDeliveryFamilyKind,
};

fn inspect_live_view_posture<T>(
    runtime: &WorthQueryRuntime,
    view: &WorthQueryLiveView<T>,
) -> WorthQueryOrdinaryRuntimePosture {
    let inspection = runtime.inspect(view).expect("inspection should succeed");
    let WorthQueryInspection::LiveView(inspection) = inspection else {
        panic!("inspection should target the live-view surface");
    };
    inspection.ordinary_runtime_posture().clone()
}

#[test]
fn runtime_state_and_inspection_share_time_only_compact_posture() {
    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            "tasks.compact-time-only",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    runtime
        .emit_time_only_delivery(
            view.name(),
            QuerySubscriptionDeliveryCauseKind::WindowEntry,
            "tick:compact-window-entry",
            false,
            true,
        )
        .expect("time-only delivery should emit");
    let _ = runtime.drain_patches(&view);

    let state = <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(
        &view, &runtime,
    )
    .expect("state should snapshot");
    let posture = state
        .ordinary_runtime_posture()
        .expect("compact posture should project on state");

    assert_eq!(
        posture.kind(),
        WorthQueryOrdinaryRuntimePostureKind::Current
    );
    assert_eq!(
        posture.cause_posture(),
        WorthQueryOrdinaryRuntimeCausePostureKind::TimeOnly
    );
    assert_eq!(posture.async_posture(), None);
    assert_eq!(
        posture.basis_posture(),
        WorthQueryOrdinaryRuntimeBasisPostureKind::Stable
    );
    assert_eq!(
        posture.support_evidence_digest(),
        view.subscription_installation()
            .support_projection()
            .label()
    );
    assert_eq!(posture, &inspect_live_view_posture(&runtime, &view));
}

#[test]
fn runtime_state_and_inspection_share_mixed_async_compact_posture() {
    let host_request = worth_runtime_bridge::facade::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

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

    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            "tasks.compact-mixed-async",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    runtime
        .emit_mixed_cause_delivery(view.name(), &ordering, &window)
        .expect("mixed-cause delivery should emit");
    let (basis_digest, generation_digest) = live_subscription_async_identity(&runtime, view.name());
    runtime
        .project_async_result_state(
            view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                "async:compact-current",
            ),
            &basis_digest,
            &generation_digest,
        )
        .expect("async result state should project");
    let _ = runtime.drain_patches(&view);

    let state = <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(
        &view, &runtime,
    )
    .expect("state should snapshot");
    let posture = state
        .ordinary_runtime_posture()
        .expect("compact posture should project on state");

    assert_eq!(
        posture.kind(),
        WorthQueryOrdinaryRuntimePostureKind::Current
    );
    assert_eq!(
        posture.cause_posture(),
        WorthQueryOrdinaryRuntimeCausePostureKind::MixedCause
    );
    assert_eq!(
        posture.async_posture(),
        Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Current)
    );
    assert_eq!(
        posture.basis_posture(),
        WorthQueryOrdinaryRuntimeBasisPostureKind::Stable
    );
    assert_eq!(
        posture.support_evidence_digest(),
        view.subscription_installation()
            .support_projection()
            .label()
    );
    assert_eq!(posture, &inspect_live_view_posture(&runtime, &view));
}

#[test]
fn runtime_compact_posture_keeps_basis_sensitive_denied_async_state_typed_on_scalar_surfaces() {
    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            "tasks.compact-async-denied",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let (basis_digest, generation_digest) = live_subscription_async_identity(&runtime, view.name());
    let drifted_basis = runtime_async_checkpoint_label_identity("basis:drifted");
    runtime
        .project_async_result_state(
            view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Denied(
                    BridgeAsyncCompletionDenialClass::SignalLifecycleDenied,
                ),
                "async:compact-denied",
            ),
            &drifted_basis,
            &generation_digest,
        )
        .expect("denied async state should project");

    let state = <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(
        &view, &runtime,
    )
    .expect("state should snapshot");
    let posture = state
        .ordinary_runtime_posture()
        .expect("compact posture should project on state");

    assert_eq!(state.kind(), WorthQueryRuntimeStateKind::Denied);
    assert_eq!(posture.kind(), WorthQueryOrdinaryRuntimePostureKind::Denied);
    assert_eq!(
        posture.cause_posture(),
        WorthQueryOrdinaryRuntimeCausePostureKind::Ordinary
    );
    assert_eq!(
        posture.async_posture(),
        Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Denied)
    );
    assert_eq!(
        posture.basis_posture(),
        WorthQueryOrdinaryRuntimeBasisPostureKind::BasisDrift
    );
    assert_ne!(basis_digest, drifted_basis);
    assert_eq!(
        posture.support_evidence_digest(),
        view.subscription_installation()
            .support_projection()
            .label()
    );
    assert_eq!(posture, &inspect_live_view_posture(&runtime, &view));
}
