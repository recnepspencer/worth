use crate::authoring::{AspectFieldSelector, AuthoredResultShapeField, OrderingSelector};
use crate::projection_consumption::ProjectionMaterializedFactPostureKind;
use crate::runtime::tests::support::{
    live_subscription_async_identity, stateful_bridge_task_runtime, task_live_request, task_schema,
    BridgeAsyncCompletionClass, BridgeAsyncCompletionState, WorthQueryRuntimeAsyncResultProjection,
};
use crate::runtime::{
    WorthQueryLiveView, WorthQueryReadFamily, WorthQueryRuntimeAsyncResultStateKind,
    WorthQueryUnrefinedLiveShape, WorthQueryWorkspace,
};
use crate::subscription::QuerySubscriptionDeliveryCauseKind;

fn task_table_read_family(
    workspace: &mut WorthQueryWorkspace,
    family_name: &str,
) -> WorthQueryReadFamily {
    workspace
        .define_read_family(family_name, |read| {
            read.local_collection(
                "Task",
                task_schema(),
                |query| {
                    query
                        .project(
                            AspectFieldSelector::new("identity", "id")
                                .expect("identity projection should build"),
                        )
                        .project(
                            AspectFieldSelector::new("title", "value")
                                .expect("title projection should build"),
                        )
                        .order_by(
                            OrderingSelector::ascending("title", "value")
                                .expect("title ordering should build"),
                        )
                },
                |shape| {
                    shape
                        .field(
                            AuthoredResultShapeField::new("identity", "id", "identity.id")
                                .expect("identity result-shape field should build"),
                        )
                        .field(
                            AuthoredResultShapeField::new("title", "value", "title")
                                .expect("title result-shape field should build"),
                        )
                },
            )
        })
        .expect("task table read family should define")
}

#[test]
fn runtime_read_family_receipt_retains_time_only_materialized_fact_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let expected_query_digest = view
        .subscription_installation()
        .query_projection()
        .label()
        .to_string();

    runtime
        .emit_time_only_delivery(
            view.name(),
            QuerySubscriptionDeliveryCauseKind::FreshnessOnly,
            "tick:read-family-time-only",
            false,
            true,
        )
        .expect("time-only delivery should project");

    let mut workspace = runtime
        .workspace("runtime.read-composition.phase22.time-only")
        .expect("workspace should open");
    let family = task_table_read_family(&mut workspace, "phase22-time-only");
    assert_eq!(
        family.read_graph().declarative_request(),
        &task_live_request()
    );
    let result = workspace
        .execute_read_family(request_execution, &family)
        .expect("read family should execute");
    assert_eq!(
        result.receipt().query_digest(),
        family.read_graph().query_digest()
    );
    let posture = result
        .receipt()
        .materialized_fact_posture()
        .expect("time-only read receipt should retain posture");

    assert_eq!(
        result.receipt().query_digest(),
        family.read_graph().query_digest()
    );
    let expected_posture_basis = result
        .receipt()
        .snapshot_identity()
        .evidence_identity()
        .as_str()
        .to_string();
    assert_eq!(
        posture.basis_digest(),
        expected_posture_basis,
        "time-only posture basis should match receipt snapshot evidence"
    );
    assert_eq!(
        posture.kind(),
        ProjectionMaterializedFactPostureKind::TimeOnly
    );
    assert_eq!(posture.lower_declaration_digest(), expected_query_digest);
    assert_eq!(posture.basis_digest(), expected_posture_basis);
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
fn runtime_read_family_receipt_retains_async_backed_materialized_fact_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut runtime = stateful_bridge_task_runtime(request_execution);
    let view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view(
            request_execution,
            "tasks.async-table",
            task_live_request(),
            task_schema(),
        )
        .expect("live view should declare");
    let expected_query_digest = view
        .subscription_installation()
        .query_projection()
        .label()
        .to_string();
    let (basis_digest, generation_digest) = live_subscription_async_identity(&runtime, view.name());

    runtime
        .project_async_result_state(
            view.name(),
            &WorthQueryRuntimeAsyncResultProjection::completion_state(
                BridgeAsyncCompletionState::Admitted(BridgeAsyncCompletionClass::Fulfilled),
                "async:read-family-current",
            ),
            &basis_digest,
            &generation_digest,
        )
        .expect("async result state should project");

    let mut workspace = runtime
        .workspace("runtime.read-composition.phase22.async")
        .expect("workspace should open");
    let family = task_table_read_family(&mut workspace, "phase22-async");
    assert_eq!(
        family.read_graph().declarative_request(),
        &task_live_request()
    );
    let result = workspace
        .execute_read_family(request_execution, &family)
        .expect("read family should execute");
    assert_eq!(
        result.receipt().query_digest(),
        family.read_graph().query_digest()
    );
    let posture = result
        .receipt()
        .materialized_fact_posture()
        .expect("async-backed read receipt should retain posture");

    assert_eq!(
        result.receipt().query_digest(),
        family.read_graph().query_digest()
    );
    let expected_posture_basis = result
        .receipt()
        .snapshot_identity()
        .evidence_identity()
        .as_str()
        .to_string();
    assert_eq!(
        posture.basis_digest(),
        expected_posture_basis,
        "async-backed posture basis should match receipt snapshot evidence"
    );
    assert_eq!(
        posture.kind(),
        ProjectionMaterializedFactPostureKind::AsyncBacked
    );
    assert_eq!(posture.lower_declaration_digest(), expected_query_digest);
    assert_eq!(posture.basis_digest(), expected_posture_basis);
    assert_eq!(
        posture.runtime_origin_digest(),
        Some(
            view.subscription_installation()
                .installation_projection()
                .label()
                .as_str()
        )
    );
    assert_eq!(
        workspace
            .state(request_execution, &view)
            .expect("state should snapshot")
            .async_result_state()
            .expect("async state should remain retained")
            .kind(),
        WorthQueryRuntimeAsyncResultStateKind::Current
    );
}
