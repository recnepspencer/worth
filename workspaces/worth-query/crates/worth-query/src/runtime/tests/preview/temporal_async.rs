use super::super::support::*;
use crate::runtime::async_result_state::runtime_async_checkpoint_label_identity;
use crate::subscription::QuerySubscriptionDeliveryCauseKind;
use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionClass, BridgeAsyncCompletionState, BridgeAsyncRequestTruthViewBasis,
    BridgeMixedCauseOrderingInput, BridgeMixedCauseOrderingLaneKind,
    BridgeMixedCauseOrderingRequest, BridgeSubscriptionDeliveryFamilyKind,
};

fn install_temporal_async_and_mixed_residue(
    runtime: &mut WorthQueryRuntime,
    temporal_view: &WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
    async_view: &WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
    mixed_view: &WorthQueryLiveView<WorthQueryUnrefinedLiveShape>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) {
    runtime
        .emit_time_only_delivery(
            temporal_view.name(),
            QuerySubscriptionDeliveryCauseKind::FreshnessOnly,
            "preview:temporal-only",
            true,
            true,
        )
        .expect("preview temporal-only residue should emit");

    let (basis_digest, generation_digest) =
        live_subscription_async_identity(runtime, async_view.name());
    runtime
        .project_async_result_state(
            async_view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                "preview:async-current",
            ),
            &basis_digest,
            &generation_digest,
        )
        .expect("preview async residue should project");

    let bridge = test_bridge();
    let truth_patch = canonical_truth_patch("truth-main", "snapshot-a", "commit-a", "patch-a");
    let truth_plus_time =
        authoritative_truth_plus_time_cause(&bridge, &truth_patch, resource_request);
    let async_completion = admitted_async_completion(
        &bridge,
        worth_signal::facade::NodeId::new(310, 0),
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
            BridgeMixedCauseOrderingInput::TruthPatch(truth_patch),
            BridgeMixedCauseOrderingInput::Temporal(truth_plus_time),
            BridgeMixedCauseOrderingInput::AsyncCompletion(async_completion),
        ],
    ));
    let window = bridge
        .plan_mixed_cause_delivery_window(
            &ordering,
            BridgeSubscriptionDeliveryFamilyKind::AdmittedCoalesced,
        )
        .expect("preview mixed-cause residue window should plan");
    runtime
        .emit_mixed_cause_delivery(mixed_view.name(), &ordering, &window)
        .expect("preview mixed-cause residue should emit");
}

#[test]
fn preview_discard_closeout_tracks_temporal_async_and_mixed_residue_parity() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime();
    let temporal_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("tasks.preview-temporal", task_live_request(), task_schema())
        .expect("temporal live view should declare");
    let async_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("tasks.preview-async", task_live_request(), task_schema())
        .expect("async live view should declare");
    let mixed_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("tasks.preview-mixed", task_live_request(), task_schema())
        .expect("mixed live view should declare");
    install_temporal_async_and_mixed_residue(
        &mut runtime,
        &temporal_view,
        &async_view,
        &mixed_view,
        resource_request,
    );

    let outcome = {
        let mut preview = runtime
            .preview(test_session_label("preview temporal-async discard"))
            .expect("preview session should admit");
        preview.use_view(&temporal_view);
        preview.use_view(&async_view);
        preview.use_view(&mixed_view);
        preview.discard()
    };

    let closeout = outcome.closeout_evidence();
    assert_eq!(closeout.temporal_wake_residue_count(), 1);
    assert_eq!(closeout.async_result_residue_count(), 1);
    assert_eq!(closeout.mixed_cause_residue_count(), 1);
    assert_eq!(closeout.crossed_authoritative_residue_count(), 0);
    assert_eq!(closeout.authoritative_residue_count(), 0);
    assert_eq!(closeout.rebinding_digest(), None);
    assert_eq!(
        closeout.preview_basis_snapshot_identity(),
        closeout.target_basis_snapshot_identity()
    );

    let inspection = runtime
        .inspect_preview_outcome(&outcome)
        .expect("preview outcome inspection should succeed");
    assert_eq!(inspection.temporal_wake_residue_count(), 1);
    assert_eq!(inspection.async_result_residue_count(), 1);
    assert_eq!(inspection.mixed_cause_residue_count(), 1);
    assert_eq!(inspection.crossed_authoritative_residue_count(), 0);
    assert_eq!(inspection.authoritative_residue_count(), 0);
    assert_eq!(inspection.rebinding_digest(), None);
    assert_eq!(
        inspection.preview_basis_snapshot_identity(),
        inspection.target_basis_snapshot_identity()
    );
}

#[test]
fn preview_promotion_closeout_records_rebinding_for_temporal_async_and_mixed_handles() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime();
    let temporal_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("tasks.promote-temporal", task_live_request(), task_schema())
        .expect("temporal live view should declare");
    let async_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("tasks.promote-async", task_live_request(), task_schema())
        .expect("async live view should declare");
    let mixed_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("tasks.promote-mixed", task_live_request(), task_schema())
        .expect("mixed live view should declare");
    runtime
        .declare_live_view::<WorthQueryUnrefinedLiveShape>(
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("authoritative target live view should declare");
    install_temporal_async_and_mixed_residue(
        &mut runtime,
        &temporal_view,
        &async_view,
        &mixed_view,
        resource_request,
    );

    let outcome = {
        let mut preview = runtime
            .preview_with_options(
                test_session_label("preview temporal-async promote"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should admit");
        preview.use_view(&temporal_view);
        preview.use_view(&async_view);
        preview.use_view(&mixed_view);
        preview
            .write(insert_command(
                "Task",
                [
                    (
                        "identity.id",
                        test_string_aspect_value("preview-promote-temporal-async"),
                    ),
                    (
                        "title.value",
                        test_string_aspect_value("Promoted preview residue task"),
                    ),
                ],
            ))
            .expect("preview write should stage");
        preview.promote().expect("preview promotion should succeed")
    };

    let closeout = outcome.closeout_evidence();
    assert!(outcome.promoted());
    assert_eq!(closeout.temporal_wake_residue_count(), 1);
    assert_eq!(closeout.async_result_residue_count(), 1);
    assert_eq!(closeout.mixed_cause_residue_count(), 1);
    assert_eq!(closeout.crossed_authoritative_residue_count(), 0);
    assert!(closeout.rebinding_digest().is_some());
    assert!(!closeout
        .preview_basis_snapshot_identity()
        .evidence_identity()
        .as_str()
        .is_empty());
    assert!(!closeout
        .target_basis_snapshot_identity()
        .evidence_identity()
        .as_str()
        .is_empty());

    let inspection = runtime
        .inspect_preview_outcome(&outcome)
        .expect("preview outcome inspection should succeed");
    assert!(inspection.rebinding_digest().is_some());
    assert_eq!(
        inspection.preview_basis_snapshot_identity(),
        closeout.preview_basis_snapshot_identity()
    );
    assert_eq!(
        inspection.target_basis_snapshot_identity(),
        closeout.target_basis_snapshot_identity()
    );
    assert_eq!(inspection.temporal_wake_residue_count(), 1);
    assert_eq!(inspection.async_result_residue_count(), 1);
    assert_eq!(inspection.mixed_cause_residue_count(), 1);
}

#[test]
fn preview_discard_retains_crossed_preview_completion_residue_typed() {
    let host_request = crate::bridge_host_request::serial_request();
    let resource_request = worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            "tasks.preview-crossed-completion",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let bridge = test_bridge();
    let truth_patch = canonical_truth_patch("truth-main", "snapshot-a", "commit-a", "patch-a");
    let preview_cause =
        preview_time_only_cause(&bridge, "preview-crossed-completion", resource_request);
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
        .expect("crossed preview delivery window should plan");
    runtime
        .emit_mixed_cause_delivery(view.name(), &ordering, &window)
        .expect("crossed preview delivery should emit");

    let outcome = {
        let mut preview = runtime
            .preview(test_session_label("preview crossed completion discard"))
            .expect("preview session should admit");
        preview.use_view(&view);
        preview.discard()
    };

    let closeout = outcome.closeout_evidence();
    assert_eq!(closeout.crossed_authoritative_residue_count(), 1);
    assert_eq!(closeout.authoritative_residue_count(), 1);
    assert_eq!(
        closeout.class_count(WorthQueryPreviewResidueClass::CrossedAuthoritativeResidue),
        1
    );

    let inspection = runtime
        .inspect_preview_outcome(&outcome)
        .expect("preview outcome inspection should succeed");
    assert_eq!(inspection.crossed_authoritative_residue_count(), 1);
    assert_eq!(inspection.authoritative_residue_count(), 1);
}

#[test]
fn preview_promotion_denies_with_typed_rebinding_recovery_posture() {
    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            "tasks.preview-promotion-mismatch",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let (_, generation_digest) = live_subscription_async_identity(&runtime, view.name());
    runtime
        .project_async_result_state(
            view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Denied(
                    worth_runtime_bridge::facade::BridgeAsyncCompletionDenialClass::SignalLifecycleDenied,
                ),
                "async:preview-mismatch",
            ),
            &runtime_async_checkpoint_label_identity("basis:drifted"),
            &generation_digest,
        )
        .expect("preview mismatch should remain typed");

    let error = {
        let mut preview = runtime
            .preview_with_options(
                test_session_label("preview promotion mismatch"),
                WorthQueryPreviewOptions::sandboxed_write_intent(),
            )
            .expect("preview session should admit");
        preview.use_view(&view);
        preview
            .write(insert_command(
                "Task",
                [
                    (
                        "identity.id",
                        test_string_aspect_value("preview-promotion-mismatch"),
                    ),
                    (
                        "title.value",
                        test_string_aspect_value("Should require rebinding"),
                    ),
                ],
            ))
            .expect("preview write should stage");
        preview
            .promote()
            .expect_err("crossed preview residue should require rebinding")
    };

    match error {
        WorthQueryRuntimeError::PreviewPromotionRebindingRequired(evidence) => {
            assert_eq!(
                evidence.kind(),
                WorthQueryPreviewPromotionDenialKind::RebindingRequired
            );
            assert_eq!(evidence.crossed_authoritative_residue_count(), 1);
            assert_eq!(
                evidence.recovery_posture(),
                "discard_preview_and_readmit_authoritative"
            );
            assert_eq!(
                evidence.denial_identity().as_str(),
                evidence.denial_digest()
            );
            assert_eq!(
                evidence
                    .rebinding_identity()
                    .map(|identity| identity.as_str()),
                evidence.rebinding_digest()
            );
            assert!(evidence.rebinding_digest().is_some());
        }
        other => panic!("expected preview rebinding denial, got {other:?}"),
    }
}
