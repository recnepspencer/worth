use super::super::support::*;

#[test]
fn workspace_delete_receipt_preserves_family_target_and_inspection_posture() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.aspect-delete")
        .expect("task runtime should open a named workspace");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.delete-table", |q| {
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
                .schema_basis("tasks-delete-table")
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
        })
        .expect("seed insert should execute");
    let _ = workspace.observe(&live);

    let receipt = workspace
        .delete(request_execution, seed.deltas()[0].entity_identity.clone())
        .expect("delete should execute");
    let state = workspace
        .state(request_execution, &receipt)
        .expect("delete receipt should expose state posture");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("delete receipt should inspect");

    assert_eq!(receipt.mutation_family(), WorthQueryMutationFamily::Delete);
    assert_eq!(receipt.terminal_declared_collection_projection(), None);
    assert_eq!(
        receipt.declared_entity_identity(),
        Some(&seed.deltas()[0].entity_identity)
    );
    assert_eq!(
        receipt.terminal_target_collection_projection(),
        Some("Task")
    );
    assert_eq!(
        receipt.target_entity_identity(),
        Some(&seed.deltas()[0].entity_identity)
    );
    assert_eq!(
        receipt.target_evidence().declared().target_class(),
        WorthQueryMutationTargetClass::Entity
    );
    assert_eq!(
        receipt.target_evidence().resolved().target_class(),
        WorthQueryMutationTargetClass::Entity
    );
    assert!(receipt.causality_evidence().is_some());
    assert!(receipt.provenance_evidence().is_some());
    assert_eq!(
        receipt.authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        receipt.basis_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(state.kind(), WorthQueryRuntimeStateKind::Ready);

    match inspection {
        WorthQueryInspection::WriteReceipt(inspection) => {
            assert_eq!(inspection.mutation_family(), "delete");
            assert_eq!(
                inspection.basis_lane(),
                WorthQueryAuthorityLane::AuthoritativeTruth
            );
            assert_eq!(inspection.target_collection(), Some("Task"));
            assert_eq!(
                inspection.target_entity_identity(),
                Some(&seed.deltas()[0].entity_identity)
            );
            assert_eq!(
                inspection.declared_entity_identity(),
                Some(&seed.deltas()[0].entity_identity)
            );
            assert!(inspection.causality_evidence().is_some());
            assert!(inspection.provenance_evidence().is_some());
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
}

#[test]
fn workspace_delete_with_preserves_touched_aspects_and_metadata_for_routing() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.aspect-delete-with")
        .expect("task runtime should open a named workspace");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.delete-with-table", |q| {
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
                .schema_basis("tasks-delete-with-table")
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
        })
        .expect("seed insert should execute");
    let _ = workspace.observe(&live);

    let receipt = workspace
        .delete_with(
            request_execution,
            seed.deltas()[0].entity_identity.clone(),
            |delete| {
                delete
                    .touch(test_aspect_touch("title.value"))
                    .metadata("author", "worth-topo")
            },
        )
        .expect("delete with declared meaning should execute");
    let patches = workspace.observe(&live);
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("delete-with receipt should inspect");

    assert_eq!(receipt.mutation_family(), WorthQueryMutationFamily::Delete);
    assert_eq!(receipt.terminal_declared_collection_projection(), None);
    assert_eq!(
        receipt.deltas()[0].admitted_touched_aspects(),
        test_aspect_touches(["title.value"]).as_slice()
    );
    assert_eq!(
        receipt
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
    assert_eq!(
        receipt.terminal_affected_live_view_ids_projection(),
        &["tasks.delete-with-table".to_string()]
    );
    assert_eq!(patches.query_delivery_batches.len(), 1);

    match inspection {
        WorthQueryInspection::WriteReceipt(inspection) => {
            assert_eq!(inspection.declared_collection(), None);
            assert_eq!(
                inspection
                    .mutation_metadata()
                    .get(&test_mutation_metadata_key("author"))
                    .map(|value| value.terminal_digest_text()),
                Some("worth-topo")
            );
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
}

#[test]
fn preview_delete_with_preserves_declared_target_collection_and_delete_meaning() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.preview-delete-with")
        .expect("task runtime should open a named workspace");
    let _: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.preview-delete-with-table", |q| {
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
                .schema_basis("tasks-preview-delete-with-table")
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
                test_authored_string_aspect_value("Preview delete target"),
            )
        })
        .expect("seed insert should execute");
    let mut preview = workspace
        .preview_with_options(
            request_execution,
            test_session_label("task-preview-delete-with"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("preview should open");

    let receipt = preview
        .delete_with(
            request_execution,
            seed.deltas()[0].entity_identity.clone(),
            |delete| {
                delete
                    .target_collection("Task")
                    .touch(test_aspect_touch("title.value"))
                    .metadata("author", "worth-topo")
            },
        )
        .expect("preview delete with declared target should stage");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("preview delete-with receipt should inspect");

    assert_eq!(
        receipt.authority_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );
    assert_eq!(
        receipt.terminal_declared_collection_projection(),
        Some("Task")
    );
    assert_eq!(
        receipt.terminal_target_collection_projection(),
        Some("Task")
    );
    assert_eq!(
        receipt
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
            assert_eq!(inspection.declared_collection(), Some("Task"));
            assert_eq!(inspection.target_collection(), Some("Task"));
            assert_eq!(
                inspection
                    .mutation_metadata()
                    .get(&test_mutation_metadata_key("author"))
                    .map(|value| value.terminal_digest_text()),
                Some("worth-topo")
            );
        }
        other => panic!("expected write receipt inspection, got {other:?}"),
    }
}

#[test]
fn workspace_batch_routes_shared_computeds_once_per_batch_boundary() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.batch-computed-routing")
        .expect("task runtime should open a named workspace");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.batch-routing-table", |q| {
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
                .schema_basis("tasks-batch-routing-table")
        })
        .expect("live view should declare");
    let _computed: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
        .computed(
            request_execution,
            "tasks.batch-routing-summary",
            |c| {
                c.depends_on_live(&live)
                    .reads(test_aspect_touches(["title.value"]))
                    .produces(test_aspect_touches(["ui.batch_routing_summary"]))
            },
            TitleListMaintainer,
        )
        .expect("computed view should declare");

    let receipt = workspace
        .batch(request_execution, |batch| {
            batch
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("task-1"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Buy milk"),
                    )
                })
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("task-2"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Buy bread"),
                    )
                })
        })
        .expect("aspect-native batch should execute");

    assert_eq!(
        receipt.considered_computed_view_count(),
        1,
        "shared computed surfaces should route once per batch boundary, not once per sub-write"
    );
}
