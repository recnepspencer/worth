use super::support::*;
use crate::projection_consumption::ProjectionMaterializedFactPostureKind;
use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionClass, BridgeAsyncCompletionState, BridgeAsyncRequestTruthViewBasis,
    BridgeMixedCauseOrderingInput, BridgeMixedCauseOrderingLaneKind,
    BridgeMixedCauseOrderingRequest, BridgeSubscriptionDeliveryFamilyKind,
};

#[test]
fn runtime_write_denies_when_signal_routing_receipt_drifts_from_write_receipt() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(DriftingSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native live receipt contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("backend with drifting signal receipt should still build");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("external.tasks", task_live_request(), task_schema())
        .expect("live view should declare before hostile write");

    let error = runtime
        .write(insert_command(
            "Task",
            [
                ("identity.id", test_string_aspect_value("")),
                (
                    "title.value",
                    test_string_aspect_value("Signal receipt drift"),
                ),
            ],
        ))
        .expect_err("signal routing receipt drift must deny the write");

    assert_workspace_write_error(
        error,
        "signal invalidation routing receipt drifted from write receipt",
    );
    assert!(runtime
        .drain_patches(&view)
        .query_delivery_batches
        .is_empty());
}

#[test]
fn runtime_write_denies_authority_less_receipt_at_signal_routing_boundary() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(AuthorityLessWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native live receipt contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("backend with authority-less write authority should build");

    let error = runtime
        .write(insert_command(
            "Task",
            [
                ("identity.id", test_string_aspect_value("authority-less")),
                (
                    "title.value",
                    test_string_aspect_value("Authority-less receipt"),
                ),
            ],
        ))
        .expect_err("authority-less write receipt must fail signal routing");

    assert_workspace_write_error(
        error,
        "signal invalidation routing requires bridge-authored mutation authority",
    );
}

#[test]
fn runtime_batch_write_denies_when_signal_routing_batch_width_drifts_from_receipts() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TruncatingBatchSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native live receipt contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("backend with truncating signal batch sink should still build");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("external.tasks.batch", task_live_request(), task_schema())
        .expect("live view should declare before hostile batch write");

    let error = runtime
        .write_batch(vec![
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("first hostile batch write"),
                    ),
                ],
            ),
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("second hostile batch write"),
                    ),
                ],
            ),
        ])
        .expect_err("signal routing batch width drift must deny the batch write");

    assert_workspace_write_error(
        error,
        "signal invalidation routing batch width drifted from write batch",
    );
    assert!(runtime
        .drain_patches(&view)
        .query_delivery_batches
        .is_empty());
}

#[test]
fn runtime_live_read_receipt_retains_materialized_remask_posture() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(RemaskingSubscriptionActivation {
            projection: WorthQueryRuntimeRemaskProjection::remasked(
                WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
                "policy:drifted",
                "tenant-truth:stable",
                "tenant-schema:stable",
                "relationship-proof:verified",
                "schema-context:tasks",
            ),
        })
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native live receipt contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("remasked runtime should build");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("external.tasks.remask", task_live_request(), task_schema())
        .expect("live view should declare");

    let receipt = runtime
        .read_live_result(&view)
        .expect("live read should execute")
        .receipt()
        .clone();
    let posture = receipt
        .materialized_fact_posture()
        .expect("live read receipt should retain materialized posture");

    assert_eq!(
        posture.kind(),
        ProjectionMaterializedFactPostureKind::Remasked
    );
    assert_eq!(posture.lower_declaration_digest(), receipt.query_digest());
    assert_eq!(
        posture.basis_digest(),
        receipt.snapshot_evidence_identity().as_str()
    );
    assert_eq!(
        posture.runtime_origin_digest(),
        Some(
            view.subscription_installation()
                .installation_projection()
                .label()
                .as_str()
        )
    );
}

#[test]
fn runtime_live_read_receipt_retains_time_only_materialized_posture() {
    let mut runtime = stateful_bridge_task_runtime();
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            "external.tasks.time-only",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    runtime
        .emit_time_only_delivery(
            view.name(),
            crate::subscription::QuerySubscriptionDeliveryCauseKind::WindowEntry,
            "tick:live-receipt-window-entry",
            false,
            true,
        )
        .expect("time-only delivery should emit");

    let receipt = runtime
        .read_live_result(&view)
        .expect("live read should execute")
        .receipt()
        .clone();
    let posture = receipt
        .materialized_fact_posture()
        .expect("live read receipt should retain time-only posture");

    assert_eq!(
        posture.kind(),
        ProjectionMaterializedFactPostureKind::TimeOnly
    );
    assert_eq!(posture.lower_declaration_digest(), receipt.query_digest());
    assert_eq!(
        posture.basis_digest(),
        receipt.snapshot_evidence_identity().as_str()
    );
    assert_eq!(
        posture.runtime_origin_digest(),
        Some(
            view.subscription_installation()
                .installation_projection()
                .label()
                .as_str()
        )
    );
}

#[test]
fn runtime_live_read_receipt_retains_async_and_mixed_cause_posture_precedence() {
    let mut async_runtime = stateful_bridge_task_runtime();
    let async_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = async_runtime
        .declare_live_view("external.tasks.async", task_live_request(), task_schema())
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
        .read_live_result(&async_view)
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
    let truth_plus_time = authoritative_truth_plus_time_cause(&bridge, &truth_patch);
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
    let mut mixed_runtime = stateful_bridge_task_runtime();
    let mixed_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = mixed_runtime
        .declare_live_view("external.tasks.mixed", task_live_request(), task_schema())
        .expect("mixed live view should declare");
    mixed_runtime
        .emit_mixed_cause_delivery(mixed_view.name(), &ordering, &window)
        .expect("mixed-cause delivery should emit");
    let mixed_receipt = mixed_runtime
        .read_live_result(&mixed_view)
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

fn assert_workspace_write_error(error: WorthQueryRuntimeError, expected_message_fragment: &str) {
    match error {
        WorthQueryRuntimeError::Workspace(error) => {
            assert!(error.to_string().contains(expected_message_fragment));
        }
        other => panic!("expected workspace write denial, got {other:?}"),
    }
}
