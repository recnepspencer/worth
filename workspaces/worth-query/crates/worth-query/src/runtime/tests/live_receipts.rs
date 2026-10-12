use super::support::*;
use crate::projection_consumption::ProjectionMaterializedFactPostureKind;
use worth_runtime_bridge::facade::{
    BridgeAsyncCompletionClass, BridgeAsyncCompletionState, BridgeAsyncRequestTruthViewBasis,
    BridgeMixedCauseOrderingInput, BridgeMixedCauseOrderingLaneKind,
    BridgeMixedCauseOrderingRequest, BridgeSubscriptionDeliveryFamilyKind,
};

#[test]
fn runtime_write_denies_when_signal_routing_receipt_drifts_from_write_receipt() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
        .build(request_execution)
        .expect("backend with drifting signal receipt should still build");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "external.tasks",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare before hostile write");

    let error = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("")),
                    (
                        "title.value",
                        test_string_aspect_value("Signal receipt drift"),
                    ),
                ],
            ),
        )
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
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
        .build(request_execution)
        .expect("backend with authority-less write authority should build");

    let error = runtime
        .write(
            request_execution,
            insert_command(
                "Task",
                [
                    ("identity.id", test_string_aspect_value("authority-less")),
                    (
                        "title.value",
                        test_string_aspect_value("Authority-less receipt"),
                    ),
                ],
            ),
        )
        .expect_err("authority-less write receipt must fail signal routing");

    assert_workspace_write_error(
        error,
        "signal invalidation routing requires bridge-authored mutation authority",
    );
}

#[test]
fn runtime_batch_write_denies_when_signal_routing_batch_width_drifts_from_receipts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
        .build(request_execution)
        .expect("backend with truncating signal batch sink should still build");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "external.tasks.batch",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare before hostile batch write");

    let error = runtime
        .write_batch(
            request_execution,
            vec![
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
            ],
        )
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
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

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
        .build(request_execution)
        .expect("remasked runtime should build");
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "external.tasks.remask",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");

    let receipt = runtime
        .read_live_result(request_execution, &view)
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
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
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
        .read_live_result(request_execution, &view)
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

#[path = "live_receipts/runtime_live_read_receipt_retains_async_and_mixed_cause_posture.rs"]
mod runtime_live_read_receipt_retains_async_and_mixed_cause_posture;

fn assert_workspace_write_error(error: WorthQueryRuntimeError, expected_message_fragment: &str) {
    match error {
        WorthQueryRuntimeError::Workspace(error) => {
            assert!(error.to_string().contains(expected_message_fragment));
        }
        other => panic!("expected workspace write denial, got {other:?}"),
    }
}
