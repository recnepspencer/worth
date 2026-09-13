use super::super::support::*;
use crate::ordinary_outcome::{
    WorthQueryOrdinaryRuntimeAsyncPostureKind, WorthQueryOrdinaryRuntimeCausePostureKind,
    WorthQueryOrdinaryRuntimePostureKind, WorthQueryOrdinaryRuntimeRemaskPostureKind,
};
use crate::subscription::QuerySubscriptionDeliveryCauseKind;
use worth_runtime_bridge::facade::{BridgeAsyncCompletionClass, BridgeAsyncCompletionState};

fn remasked_runtime(projection: WorthQueryRuntimeRemaskProjection) -> WorthQueryRuntime {
    test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(RemaskingSubscriptionActivation { projection })
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .build_backend_from_parts()
        .build()
        .expect("bridge-backed runtime should build")
}

fn inspect_live_view<T>(
    runtime: &WorthQueryRuntime,
    view: &WorthQueryLiveView<T>,
) -> WorthQueryLiveViewInspection {
    let inspection = runtime.inspect(view).expect("inspection should succeed");
    let WorthQueryInspection::LiveView(inspection) = inspection else {
        panic!("inspection should target the live-view surface");
    };
    inspection
}

#[test]
fn runtime_remask_posture_projects_parity_across_state_and_inspection() {
    let mut runtime = remasked_runtime(WorthQueryRuntimeRemaskProjection::remasked(
        WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
        "policy:drifted",
        "tenant-truth:stable",
        "tenant-schema:stable",
        "relationship-proof:verified",
        "schema-context:task-table",
    ));
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("tasks.remask-policy", task_live_request(), task_schema())
        .expect("live view should declare");
    runtime
        .emit_time_only_delivery(
            view.name(),
            QuerySubscriptionDeliveryCauseKind::WindowEntry,
            "tick:remask-window-entry",
            false,
            true,
        )
        .expect("time-only delivery should emit");
    let (basis_digest, generation_digest) = live_subscription_async_identity(&runtime, view.name());
    runtime
        .project_async_result_state(
            view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                "async:remask-current",
            ),
            &basis_digest,
            &generation_digest,
        )
        .expect("async current state should project");

    let state = <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(
        &view, &runtime,
    )
    .expect("state should snapshot");
    let inspection = inspect_live_view(&runtime, &view);
    let posture = state
        .ordinary_runtime_posture()
        .expect("ordinary posture should project");
    let projected = state
        .remask_posture()
        .expect("activation-backed remask posture should project");

    assert_eq!(state.kind(), WorthQueryRuntimeStateKind::Remasked);
    assert_eq!(inspection.remask_posture(), Some(projected));
    assert_eq!(
        posture.kind(),
        WorthQueryOrdinaryRuntimePostureKind::Remasked
    );
    assert_eq!(
        posture.cause_posture(),
        WorthQueryOrdinaryRuntimeCausePostureKind::TimeOnly
    );
    assert_eq!(
        posture.async_posture(),
        Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Current)
    );
    assert_eq!(
        posture.remask_posture(),
        Some(WorthQueryOrdinaryRuntimeRemaskPostureKind::PolicyDrift)
    );
    assert_eq!(
        state
            .async_result_state()
            .expect("async state should stay retained")
            .kind(),
        WorthQueryRuntimeAsyncResultStateKind::Current
    );
}

#[test]
fn runtime_remask_denial_stays_typed_and_does_not_collapse_into_generic_async_failure() {
    for (view_name, reason, expected_posture) in [
        (
            "tasks.remask-tenant-denied",
            WorthQueryRuntimeRemaskReasonKind::TenantDrift,
            WorthQueryOrdinaryRuntimeRemaskPostureKind::TenantDrift,
        ),
        (
            "tasks.remask-proof-denied",
            WorthQueryRuntimeRemaskReasonKind::RelationshipProofDrift,
            WorthQueryOrdinaryRuntimeRemaskPostureKind::RelationshipProofDrift,
        ),
        (
            "tasks.remask-schema-denied",
            WorthQueryRuntimeRemaskReasonKind::SchemaContextDrift,
            WorthQueryOrdinaryRuntimeRemaskPostureKind::SchemaContextDrift,
        ),
    ] {
        let mut runtime = remasked_runtime(WorthQueryRuntimeRemaskProjection::denied(
            reason,
            "policy:stable",
            "tenant-truth:drifted",
            "tenant-schema:drifted",
            "relationship-proof:drifted",
            "schema-context:drifted",
        ));
        let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
            .declare_live_view(view_name, task_live_request(), task_schema())
            .expect("live view should declare");
        let (basis_digest, generation_digest) =
            live_subscription_async_identity(&runtime, view.name());
        runtime
            .project_async_result_state(
                view.name(),
                &WorthQueryRuntimeAsyncResultProjection::completion_state(
                    BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                    "async:remask-current",
                ),
                &basis_digest,
                &generation_digest,
            )
            .expect("async current state should project");

        let state =
            <&WorthQueryLiveView<WorthQueryUnrefinedLiveShape> as WorthQueryRuntimeStateTarget>::into_state_snapshot(
                &view, &runtime,
            )
            .expect("state should snapshot");
        let inspection = inspect_live_view(&runtime, &view);
        let posture = state
            .ordinary_runtime_posture()
            .expect("ordinary posture should project");
        let projected = state
            .remask_posture()
            .expect("activation-backed remask posture should project");

        assert_eq!(state.kind(), WorthQueryRuntimeStateKind::Denied);
        assert_eq!(inspection.remask_posture(), Some(projected));
        assert_eq!(posture.kind(), WorthQueryOrdinaryRuntimePostureKind::Denied);
        assert_eq!(
            posture.async_posture(),
            Some(WorthQueryOrdinaryRuntimeAsyncPostureKind::Current)
        );
        assert_eq!(posture.remask_posture(), Some(expected_posture));
        assert_eq!(
            state
                .async_result_state()
                .expect("async state should stay retained")
                .kind(),
            WorthQueryRuntimeAsyncResultStateKind::Current
        );
        assert!(state
            .explanation()
            .contains(projected.reason_kind().as_str()));
    }
}
