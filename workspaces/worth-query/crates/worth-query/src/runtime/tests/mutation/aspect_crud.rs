use super::super::support::*;

#[path = "aspect_crud/workspace_insert_uses_aspect_native_authoring_and_routes_live_de.rs"]
mod workspace_insert_uses_aspect_native_authoring_and_routes_live_de;
#[test]
fn workspace_update_supports_multi_aspect_authoring_and_narrows_by_touched_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.aspect-update")
        .expect("task runtime should open a named workspace");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.title-only", |q| {
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
                .schema_basis("tasks-title-only")
        })
        .expect("live view should declare");

    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Buy milk"),
            )
            .set_aspect(
                test_aspect_touch("description.value"),
                test_authored_string_aspect_value("whole milk"),
            )
        })
        .expect("seed insert should execute");
    let _ = workspace.observe(&live);

    let rename = workspace
        .update(
            request_execution,
            seed.deltas()[0].entity_identity.clone(),
            |task| {
                task.set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Buy oat milk"),
                )
                .set_aspect(
                    test_aspect_touch("description.value"),
                    test_authored_string_aspect_value("oat milk"),
                )
            },
        )
        .expect("multi-aspect update should execute");
    let rename_patches = workspace.observe(&live);

    assert_eq!(
        rename.deltas()[0].admitted_touched_aspects(),
        test_aspect_touches(["description.value", "title.value"]).as_slice()
    );
    assert_eq!(rename.mutation_family(), WorthQueryMutationFamily::Update);
    assert_eq!(
        rename.declared_entity_identity(),
        Some(&seed.deltas()[0].entity_identity)
    );
    assert_eq!(
        rename.terminal_affected_live_view_ids_projection(),
        &["tasks.title-only".to_string()]
    );
    assert_eq!(rename_patches.query_delivery_batches.len(), 1);
    assert_eq!(
        rename_patches.query_delivery_batches[0].patch_group_kind(),
        QueryPatchGroupKind::DetailFieldPatchGroup
    );

    let irrelevant = workspace
        .update(
            request_execution,
            seed.deltas()[0].entity_identity.clone(),
            |task| {
                task.set_aspect(
                    test_aspect_touch("description.value"),
                    test_authored_string_aspect_value("still hidden"),
                )
            },
        )
        .expect("irrelevant aspect update should still execute");
    let irrelevant_patches = workspace.observe(&live);

    assert_eq!(
        irrelevant.deltas()[0].admitted_touched_aspects(),
        test_aspect_touches(["description.value"]).as_slice()
    );
    assert!(irrelevant
        .terminal_affected_live_view_ids_projection()
        .is_empty());
    assert!(irrelevant_patches.query_delivery_batches.is_empty());
}

#[test]
fn write_receipt_inspection_retains_authored_mutation_metadata() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.aspect-metadata")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.metadata-table", |q| {
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
                .schema_basis("tasks-metadata-table")
        })
        .expect("live view should declare");

    let receipt = workspace
        .insert(request_execution, "Task", |task| {
            task.metadata("author", "worth-topo")
                .metadata("intent", "topology-refresh")
                .set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-1"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Metadata receipt"),
                )
        })
        .expect("aspect-native insert should execute");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("write receipt should inspect");

    match inspection {
        WorthQueryInspection::WriteReceipt(inspection) => {
            assert_eq!(
                inspection
                    .mutation_metadata()
                    .get(&test_mutation_metadata_key("author"))
                    .map(|value| value.terminal_digest_text()),
                Some("worth-topo")
            );
            assert_eq!(
                inspection
                    .mutation_metadata()
                    .get(&test_mutation_metadata_key("intent"))
                    .map(|value| value.terminal_digest_text()),
                Some("topology-refresh")
            );
            assert_eq!(
                inspection.target_evidence().declared().target_class(),
                WorthQueryMutationTargetClass::Collection
            );
            assert!(inspection.causality_evidence().is_some());
            assert!(inspection.provenance_evidence().is_some());
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
}

#[test]
fn workspace_update_clear_supports_typed_reset_without_waking_unrelated_surfaces() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.aspect-clear")
        .expect("task runtime should open a named workspace");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.clear-title-only", |q| {
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
                .schema_basis("tasks-clear-title-only")
        })
        .expect("live view should declare");

    let seed = workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Buy milk"),
            )
            .set_aspect(
                test_aspect_touch("description.value"),
                test_authored_string_aspect_value("whole milk"),
            )
        })
        .expect("seed insert should execute");
    let _ = workspace.observe(&live);

    let hidden_clear = workspace
        .update(
            request_execution,
            seed.deltas()[0].entity_identity.clone(),
            |task| task.clear(test_aspect_touch("description.value")),
        )
        .expect("typed clear should execute");
    let hidden_patches = workspace.observe(&live);

    assert_eq!(
        hidden_clear.deltas()[0].admitted_touched_aspects(),
        test_aspect_touches(["description.value"]).as_slice()
    );
    assert!(hidden_patches.query_delivery_batches.is_empty());

    let visible_clear = workspace
        .update(
            request_execution,
            seed.deltas()[0].entity_identity.clone(),
            |task| task.clear(test_aspect_touch("title.value")),
        )
        .expect("clearing a projected aspect should execute");
    let visible_patches = workspace.observe(&live);
    let rows = workspace.read(request_execution, &live);
    let inspection = workspace
        .inspect(request_execution, &visible_clear)
        .expect("clear receipt should inspect");

    assert_eq!(
        visible_clear.deltas()[0].admitted_touched_aspects(),
        test_aspect_touches(["title.value"]).as_slice()
    );
    assert_eq!(visible_patches.query_delivery_batches.len(), 1);
    assert_eq!(
        visible_clear
            .declared_aspect_operations()
            .iter()
            .map(|operation| {
                format!(
                    "{}:{}",
                    operation.kind(),
                    operation.aspect_touch().admitted_touch_digest_part()
                )
            })
            .collect::<Vec<_>>(),
        vec!["clear:title:value".to_string()]
    );
    match inspection {
        WorthQueryInspection::WriteReceipt(inspection) => {
            assert_eq!(
                inspection
                    .declared_aspect_operations()
                    .iter()
                    .map(|operation| {
                        format!(
                            "{}:{}",
                            operation.kind(),
                            operation.aspect_touch().admitted_touch_digest_part()
                        )
                    })
                    .collect::<Vec<_>>(),
                vec!["clear:title:value".to_string()]
            );
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
    assert_eq!(test_native_scalar_value(&rows[0], "title.value"), None);
}
