use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
use worth_query::facade::runtime::{
    QueryPatchGroupKind, WorthQueryLiveView, WorthQueryUnrefinedLiveShape,
};

use crate::support;

use support::aspect_touch as touch;
use support::public_bridge_runtime::{
    public_bridge_runtime_bootstrap_invocation_count,
    reset_public_bridge_runtime_bootstrap_invocations, PublicBridgeRuntimeBootstrapPath,
    PublicBridgeRuntimeHarness,
};

#[test]
fn raw_runtime_read_bootstrap_simplicity_test() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("public.runtime-backed-read-bootstrap")
        .expect("runtime should open a named workspace");
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "public.runtime-backed-read-bootstrap.tasks",
            |q| {
                q.from("Task")
                    .select([
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "identity", "id",
                        )
                        .unwrap(),
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    )
                    .schema_basis("public-runtime-backed-read-bootstrap-tasks")
            },
        )
        .expect("task live view should declare");

    workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-bootstrap"))
                .set_aspect(touch("title.value"), authored_text("Bootstrap task"))
        })
        .expect("insert should execute through the public bootstrap lane");

    let rows = workspace.read(request_execution, &tasks);
    let patches = workspace.observe(&tasks);

    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].scalar_value_at(&field_path("identity.id")),
        Some(&text("task-bootstrap"))
    );
    assert_eq!(
        rows[0].scalar_value_at(&field_path("title.value")),
        Some(&text("Bootstrap task"))
    );
    assert_eq!(patches.query_delivery_batches.len(), 1);
    assert_eq!(
        patches.query_delivery_batches[0].patch_group_kind(),
        QueryPatchGroupKind::CollectionMembershipPatchGroup
    );
    assert_eq!(patches.query_delivery_batches[0].sequence(), 1);
    assert!(patches.query_delivery_batches[0].has_relational_patch());
}

#[test]
fn runtime_backed_read_bootstrap_observe_is_a_drain_not_a_snapshot() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("public.runtime-backed-read-bootstrap-drain")
        .expect("runtime should open a named workspace");
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "public.runtime-backed-read-bootstrap-drain.tasks",
            |q| {
                q.from("Task")
                    .select([
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "identity", "id",
                        )
                        .unwrap(),
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    )
                    .schema_basis("public-runtime-backed-read-bootstrap-drain-tasks")
            },
        )
        .expect("task live view should declare");

    workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-bootstrap-drain"))
                .set_aspect(touch("title.value"), authored_text("Drain bootstrap task"))
        })
        .expect("insert should execute through the public bootstrap lane");

    let first = workspace.observe(&tasks);
    let second = workspace.observe(&tasks);

    assert_eq!(first.query_delivery_batches.len(), 1);
    assert!(second.query_delivery_batches.is_empty());
}

#[test]
fn runtime_backed_read_bootstrap_narrows_observation_to_touched_projection_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("public.runtime-backed-read-bootstrap-narrowing")
        .expect("runtime should open a named workspace");
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "public.runtime-backed-read-bootstrap-narrowing.tasks",
            |q| {
                q.from("Task")
                    .select([
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "identity", "id",
                        )
                        .unwrap(),
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    )
                    .schema_basis("public-runtime-backed-read-bootstrap-narrowing-tasks")
            },
        )
        .expect("task live view should declare");

    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                touch("identity.id"),
                authored_text("task-bootstrap-narrowing"),
            )
            .set_aspect(touch("title.value"), authored_text("Bootstrap title"))
            .set_aspect(
                touch("description.value"),
                authored_text("hidden description"),
            )
        })
        .expect("seed insert should execute through the public bootstrap lane");
    let _ = workspace.observe(&tasks);

    workspace
        .update(
            request_execution,
            seed.deltas()[0].entity_identity().clone(),
            |task| {
                task.set_aspect(
                    touch("title.value"),
                    authored_text("Bootstrap title updated"),
                )
                .set_aspect(touch("description.value"), authored_text("still hidden"))
            },
        )
        .expect("projected update should execute through the public bootstrap lane");
    let projected = workspace.observe(&tasks);

    workspace
        .update(
            request_execution,
            seed.deltas()[0].entity_identity().clone(),
            |task| task.set_aspect(touch("description.value"), authored_text("hidden again")),
        )
        .expect("hidden-only update should execute through the public bootstrap lane");
    let hidden_only = workspace.observe(&tasks);
    let rows = workspace.read(request_execution, &tasks);

    assert_eq!(projected.query_delivery_batches.len(), 1);
    assert_eq!(
        projected.query_delivery_batches[0].patch_group_kind(),
        QueryPatchGroupKind::DetailFieldPatchGroup
    );
    assert!(hidden_only.query_delivery_batches.is_empty());
    assert_eq!(
        rows[0].scalar_value_at(&field_path("title.value")),
        Some(&text("Bootstrap title updated"))
    );
}

#[test]
fn runtime_backed_read_bootstrap_removes_deleted_members_from_read_and_observe() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("public.runtime-backed-read-bootstrap-delete")
        .expect("runtime should open a named workspace");
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "public.runtime-backed-read-bootstrap-delete.tasks",
            |q| {
                q.from("Task")
                    .select([
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "identity", "id",
                        )
                        .unwrap(),
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    ])
                    .order_by(
                        worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                            "title", "value",
                        )
                        .unwrap(),
                    )
                    .schema_basis("public-runtime-backed-read-bootstrap-delete-tasks")
            },
        )
        .expect("task live view should declare");

    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-bootstrap-delete"))
                .set_aspect(touch("title.value"), authored_text("Delete bootstrap task"))
        })
        .expect("seed insert should execute through the public bootstrap lane");
    let _ = workspace.observe(&tasks);

    workspace
        .delete(
            request_execution,
            seed.deltas()[0].entity_identity().clone(),
        )
        .expect("delete should execute through the public bootstrap lane");

    let patches = workspace.observe(&tasks);
    let rows = workspace.read(request_execution, &tasks);

    assert_eq!(patches.query_delivery_batches.len(), 1);
    assert_eq!(
        patches.query_delivery_batches[0].patch_group_kind(),
        QueryPatchGroupKind::CollectionMembershipPatchGroup
    );
    assert!(rows.is_empty());
}

#[test]
fn ordinary_runtime_backed_read_bootstrap_stays_on_common_lane_without_builder_scaffolding() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    reset_public_bridge_runtime_bootstrap_invocations();

    let harness = PublicBridgeRuntimeHarness::new();
    let _runtime = harness.bridge_backed_runtime(request_execution);

    assert_eq!(
        public_bridge_runtime_bootstrap_invocation_count(PublicBridgeRuntimeBootstrapPath::Common),
        1
    );
    assert_eq!(
        public_bridge_runtime_bootstrap_invocation_count(PublicBridgeRuntimeBootstrapPath::Builder),
        0
    );
}

fn authored_text(
    value: impl Into<String>,
) -> worth_query::facade::runtime::WorthQueryAuthoredAspectValue {
    worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(value)
}

fn text(value: impl Into<String>) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(value.into().into())
}

fn field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.').map(|segment| {
            FieldKey::new(segment).expect("test field path segment should be valid")
        }),
    )
    .expect("test field path should be non-empty")
}
