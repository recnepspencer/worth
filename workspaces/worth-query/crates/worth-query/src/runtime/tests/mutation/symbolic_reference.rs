use super::super::support::*;

#[path = "symbolic_reference/mixed_batch_symbolic_and_existing_targets_preserve_distinct_evid.rs"]
mod mixed_batch_symbolic_and_existing_targets_preserve_distinct_evid;

#[test]
fn symbolic_target_reference_denies_missing_same_batch_target() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.symbolic-denial")
        .expect("task runtime should open a named workspace");
    let symbolic = WorthQuerySymbolicTargetReference::new("missing-task")
        .expect("symbolic reference should build")
        .in_target_collection("Task")
        .expect("symbolic reference collection should build");

    let error = workspace
        .batch(request_execution, |batch| {
            batch.update_symbolic(symbolic, |task| {
                task.set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("No target"),
                )
            })
        })
        .expect_err("missing same-batch symbolic target should deny");

    match error {
        WorthQueryRuntimeError::MutationTargetReferenceDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQuerySymbolicTargetReferenceDenialKind::UnresolvedSameBatchTarget
            );
            assert!(!denial.denial_digest().is_empty());
        }
        other => panic!("expected symbolic target denial, got {other:?}"),
    }
}

#[test]
fn symbolic_target_reference_denies_collection_mismatch_with_native_target_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.symbolic-collection-mismatch")
        .expect("task runtime should open a named workspace");
    let symbolic = WorthQuerySymbolicTargetReference::new("draft-task")
        .expect("symbolic reference should build")
        .in_target_collection("Project")
        .expect("symbolic reference collection should build");

    let error = workspace
        .batch(request_execution, |batch| {
            batch
                .insert_symbolic("draft-task", "Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("task-draft"),
                    )
                })
                .update_symbolic(symbolic, |task| {
                    task.set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Wrong collection"),
                    )
                })
        })
        .expect_err("collection-mismatched same-batch symbolic target should deny");

    match error {
        WorthQueryRuntimeError::MutationTargetReferenceDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQuerySymbolicTargetReferenceDenialKind::CollectionMismatch
            );
            assert!(denial.message().contains("resolved to collection `Task`"));
            assert!(denial.message().contains("not `Project`"));
        }
        other => panic!("expected symbolic target denial, got {other:?}"),
    }
}

#[test]
fn preview_batch_symbolic_target_preserves_symbolic_evidence() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = stateful_bridge_task_runtime(request_execution);
    let mut workspace = runtime
        .workspace("tasks.preview-symbolic")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.preview-symbolic-table", |q| {
            q.from("Task")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                )
                .schema_basis("tasks-preview-symbolic-table")
        })
        .expect("live view should declare");

    let mut preview = workspace
        .preview(request_execution, test_session_label("symbolic-preview"))
        .expect("preview should open");
    let receipt = preview
        .batch(request_execution, |batch| {
            batch
                .insert_symbolic("draft-task", "Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("task-draft"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Draft"),
                    )
                })
                .update_symbolic(
                    WorthQuerySymbolicTargetReference::new("draft-task")
                        .expect("symbolic reference should build")
                        .in_target_collection("Task")
                        .expect("symbolic reference collection should build"),
                    |task| {
                        task.set_aspect(
                            test_aspect_touch("title.value"),
                            test_authored_string_aspect_value("Draft preview renamed"),
                        )
                    },
                )
        })
        .expect("preview batch should execute");

    assert_eq!(
        receipt
            .batch_mutation_evidence()
            .symbolic_target_reference_count(),
        1
    );
    assert_eq!(
        receipt.write_receipts()[1]
            .symbolic_target_reference_evidence()
            .expect("preview component should retain symbolic evidence")
            .symbol()
            .as_str(),
        "draft-task"
    );
}

#[test]
fn symbolic_aspect_reference_requires_batch_context() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_edge_runtime(request_execution)
        .workspace("tasks.symbolic-aspect-single")
        .expect("runtime should open a named workspace");
    let command = WorthQueryAspectMutationBuilder::new()
        .set_aspect(
            test_aspect_touch("edge.kind"),
            test_authored_string_aspect_value("depends_on"),
        )
        .symbolic_entity_identity(
            test_aspect_touch("edge.source_identity"),
            WorthQuerySymbolicTargetReference::new("draft-task")
                .expect("symbolic reference should build"),
        )
        .existing_entity_identity(
            test_aspect_touch("edge.target_identity"),
            test_entity_identity("task-existing"),
        )
        .build_insert("TaskEdge")
        .expect("insert command should build");

    let error = workspace
        .write(request_execution, command)
        .expect_err("symbolic aspect references must fail closed outside batch execution");

    match error {
        WorthQueryRuntimeError::MutationTargetReferenceDenied(denial) => {
            assert_eq!(
                denial.kind(),
                WorthQuerySymbolicTargetReferenceDenialKind::RequiresBatchContext
            );
        }
        other => panic!("expected symbolic aspect batch-context denial, got {other:?}"),
    }
}

#[test]
fn symbolic_aspect_reference_resolves_same_batch_created_entity_identity() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_edge_runtime(request_execution)
        .workspace("tasks.symbolic-aspect-batch")
        .expect("runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.symbolic-aspect-tasks", |q| {
            q.from("Task")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("identity", "id")
                        .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("title", "value")
                        .unwrap(),
                )
                .schema_basis("tasks-symbolic-aspect-tasks")
        })
        .expect("task live view should declare");
    let edges: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.symbolic-aspect-edges", |q| {
            q.from("TaskEdge")
                .select([
                    crate::authoring::AspectFieldKey::from_authoring_parts("edge", "kind").unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts(
                        "edge",
                        "source_identity",
                    )
                    .unwrap(),
                    crate::authoring::AspectFieldKey::from_authoring_parts(
                        "edge",
                        "target_identity",
                    )
                    .unwrap(),
                ])
                .order_by(
                    crate::authoring::AspectFieldKey::from_authoring_parts("edge", "kind").unwrap(),
                )
                .schema_basis("tasks-symbolic-aspect-edges")
        })
        .expect("edge live view should declare");

    let receipt = workspace
        .batch(request_execution, |batch| {
            batch
                .insert_symbolic("draft-task", "Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("task-draft"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Draft task"),
                    )
                })
                .insert("TaskEdge", |edge| {
                    edge.set_aspect(
                        test_aspect_touch("edge.kind"),
                        test_authored_string_aspect_value("depends_on"),
                    )
                    .symbolic_entity_identity(
                        test_aspect_touch("edge.source_identity"),
                        WorthQuerySymbolicTargetReference::new("draft-task")
                            .expect("symbolic reference should build"),
                    )
                    .existing_entity_identity(
                        test_aspect_touch("edge.target_identity"),
                        test_entity_identity("task-existing"),
                    )
                })
        })
        .expect("symbolic aspect batch should execute");

    let draft_identity = receipt.write_receipts()[0].deltas()[0]
        .entity_identity
        .clone();
    let edge_rows = workspace.read(request_execution, &edges);
    assert_eq!(edge_rows.len(), 1);
    let expected_source = test_native_entity_ref_value(&draft_identity);
    let expected_target = test_native_entity_ref_value(&test_entity_identity("task-existing"));
    assert_eq!(
        test_native_scalar_value(&edge_rows[0], "edge.source_identity"),
        Some(&expected_source)
    );
    assert_eq!(
        test_native_scalar_value(&edge_rows[0], "edge.target_identity"),
        Some(&expected_target)
    );
}
