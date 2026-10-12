use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey};
use worth_query::facade::consumer_kit::{
    compare_test_backend_write_receipts, in_memory_test_runtime,
};
use worth_query::facade::foundation::WorthQueryWorkspaceErrorKind;
use worth_query::facade::runtime::{InvariantCatalog, InvariantRegistration, InvariantRule};
use worth_query::facade::runtime::{
    WorthQueryRuntimeError, WorthQueryUnrefinedLiveShape, WorthQueryWriteReceipt,
};

use crate::support;

use support::aspect_touch as touch;
use support::public_bridge_runtime::PublicBridgeRuntimeHarness;

#[test]
fn in_memory_test_backend_facade_builds_a_real_workspace() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = in_memory_test_runtime()
        .with_schema(task_schema())
        .invariant_catalog(InvariantCatalog::default())
        .workspace(request_execution, "consumer-kit.facade.test")
        .expect("facade should build an in-memory test workspace");
    let tasks = workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "consumer-kit.facade.tasks",
            |view| {
                view.from("Task").select([
                    worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "identity", "id",
                    )
                    .unwrap(),
                    worth_query::facade::foundation::AspectFieldKey::from_authoring_parts(
                        "title", "value",
                    )
                    .unwrap(),
                ])
            },
        )
        .expect("facade workspace should declare a live view");

    workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-1"))
                .set_aspect(touch("title.value"), authored_text("Facade proof"))
        })
        .expect("facade workspace should write through the real runtime");

    assert_eq!(workspace.read(request_execution, &tasks).len(), 1);
}

#[test]
fn in_memory_test_backend_facade_admits_preview_and_invariant_denial() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let schema = task_schema();
    let mut preview_workspace = in_memory_test_runtime()
        .with_schema(schema.clone())
        .workspace(request_execution, "consumer-kit.facade.preview")
        .expect("facade should build preview workspace");
    let mut preview = preview_workspace
        .preview(
            request_execution,
            worth_query::facade::runtime::WorthQuerySessionLabel::scoped_strs(
                "consumer-kit-facade",
                ["preview"],
            )
            .expect("preview label"),
        )
        .expect("facade should admit preview");
    preview
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-preview"))
                .set_aspect(touch("title.value"), authored_text("Preview facade proof"))
        })
        .expect("facade preview should stage schema-backed write");
    assert_eq!(preview.discard().authoritative_residue_count(), 0);

    let mut invariant_workspace = in_memory_test_runtime()
        .with_schema(schema)
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::commit_boundary_blocking(
                InvariantRule::MaxMergedIntents(0),
            )],
        })
        .workspace(request_execution, "consumer-kit.facade.invariant-denial")
        .expect("facade should build invariant workspace");
    let error = invariant_workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-denied"))
                .set_aspect(touch("title.value"), authored_text("Denied"))
        })
        .expect_err("facade invariant should deny through runtime write path");

    assert_workspace_error_kind(error, WorthQueryWorkspaceErrorKind::Unclassified);
}

#[test]
fn in_memory_test_backend_matches_bridge_harness_for_covered_live_write_path() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let schema = task_schema();
    let mut in_memory_workspace = in_memory_test_runtime()
        .with_schema(schema)
        .workspace(
            request_execution,
            "consumer-kit.facade.equivalence.in-memory",
        )
        .expect("in-memory facade should build a workspace");
    let bridge_harness = PublicBridgeRuntimeHarness::new();
    let bridge_runtime = bridge_harness.bridge_backed_runtime(request_execution);
    let mut bridge_workspace = bridge_runtime
        .workspace("consumer-kit.facade.equivalence.bridge")
        .expect("bridge harness should build a workspace");
    let in_memory_tasks = in_memory_workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "consumer-kit.facade.equivalence.in-memory.tasks",
            |view| {
                view.from("Task")
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
            },
        )
        .expect("in-memory facade should declare live view");
    let bridge_tasks = bridge_workspace
        .live_view::<WorthQueryUnrefinedLiveShape>(
            request_execution,
            "consumer-kit.facade.equivalence.bridge.tasks",
            |view| {
                view.from("Task")
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
            },
        )
        .expect("bridge harness should declare live view");

    let in_memory_receipt = in_memory_workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-equivalence"))
                .set_aspect(touch("title.value"), authored_text("Equivalent task"))
        })
        .expect("in-memory write should execute through runtime");
    let bridge_receipt = bridge_workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(touch("identity.id"), authored_text("task-equivalence"))
                .set_aspect(touch("title.value"), authored_text("Equivalent task"))
        })
        .expect("bridge write should execute through runtime");
    let in_memory_identity = in_memory_receipt
        .target_entity_identity()
        .expect("in-memory insert identity")
        .clone();
    let bridge_identity = bridge_receipt
        .target_entity_identity()
        .expect("bridge insert identity")
        .clone();

    let in_memory_rows = in_memory_workspace
        .read(request_execution, &in_memory_tasks)
        .into_iter()
        .map(|row| selected_task_scalars(&row))
        .collect::<Vec<_>>();
    let bridge_rows = bridge_workspace
        .read(request_execution, &bridge_tasks)
        .into_iter()
        .map(|row| selected_task_scalars(&row))
        .collect::<Vec<_>>();

    assert_eq!(in_memory_rows, bridge_rows);
    assert_test_backend_receipt_equivalence(&in_memory_receipt, &bridge_receipt);

    let in_memory_update = in_memory_workspace
        .update(request_execution, in_memory_identity.clone(), |task| {
            task.set_aspect(touch("title.value"), authored_text("Updated equivalently"))
        })
        .expect("in-memory update should execute");
    let bridge_update = bridge_workspace
        .update(request_execution, bridge_identity.clone(), |task| {
            task.set_aspect(touch("title.value"), authored_text("Updated equivalently"))
        })
        .expect("bridge update should execute");
    assert_test_backend_receipt_equivalence(&in_memory_update, &bridge_update);
    assert_eq!(
        selected_task_scalars(&in_memory_workspace.read(request_execution, &in_memory_tasks)[0]),
        selected_task_scalars(&bridge_workspace.read(request_execution, &bridge_tasks)[0])
    );

    let in_memory_preview = in_memory_workspace
        .preview(
            request_execution,
            worth_query::facade::runtime::WorthQuerySessionLabel::scoped_strs(
                "consumer-kit-facade",
                ["in-memory-equivalence-preview"],
            )
            .expect("preview label"),
        )
        .expect("in-memory preview should admit");
    let bridge_preview = bridge_workspace
        .preview(
            request_execution,
            worth_query::facade::runtime::WorthQuerySessionLabel::scoped_strs(
                "consumer-kit-facade",
                ["bridge-equivalence-preview"],
            )
            .expect("preview label"),
        )
        .expect("bridge preview should admit");
    assert_eq!(
        in_memory_preview.basis_admission().effect_policy(),
        bridge_preview.basis_admission().effect_policy()
    );

    let in_memory_delete = in_memory_workspace
        .delete(request_execution, in_memory_identity)
        .expect("in-memory delete should execute");
    let bridge_delete = bridge_workspace
        .delete(request_execution, bridge_identity)
        .expect("bridge delete should execute");
    assert_test_backend_receipt_equivalence(&in_memory_delete, &bridge_delete);
    assert!(in_memory_workspace
        .read(request_execution, &in_memory_tasks)
        .is_empty());
    assert!(bridge_workspace
        .read(request_execution, &bridge_tasks)
        .is_empty());
}

fn assert_test_backend_receipt_equivalence(
    in_memory: &WorthQueryWriteReceipt,
    bridge: &WorthQueryWriteReceipt,
) {
    let report = compare_test_backend_write_receipts(in_memory, bridge);
    assert!(report.matched(), "{:?}", report.rows());
    assert_eq!(report.rows().len(), 8);
    assert!(!report
        .report_identity()
        .terminal_projection_for_reporting()
        .is_empty());
}

fn task_schema() -> worth_query::facade::consumer_kit::WorthQueryTestBackendSchema {
    support::task_backend_schema::task_backend_schema()
}

fn selected_task_scalars(
    row: &worth_query::facade::foundation::WorthQueryEntity,
) -> Vec<(CanonicalFieldPath, Option<AspectValue>)> {
    ["identity.id", "title.value"]
        .into_iter()
        .map(|path| {
            let field_path = field_path(path);
            let value = row.scalar_value_at(&field_path).cloned();
            (field_path, value)
        })
        .collect()
}

fn field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.').map(|segment| {
            FieldKey::new(segment).expect("test field path segment should be valid")
        }),
    )
    .expect("test field path should be non-empty")
}

fn authored_text(
    value: impl Into<String>,
) -> worth_query::facade::runtime::WorthQueryAuthoredAspectValue {
    worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(value)
}

fn assert_workspace_error_kind(error: WorthQueryRuntimeError, kind: WorthQueryWorkspaceErrorKind) {
    match error {
        WorthQueryRuntimeError::Workspace(workspace_error) => {
            assert_eq!(workspace_error.kind(), kind, "{workspace_error}");
        }
        other => panic!("expected workspace error `{kind:?}`, got {other:?}"),
    }
}
