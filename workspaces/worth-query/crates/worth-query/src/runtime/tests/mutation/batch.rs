use super::super::support::*;

#[test]
fn workspace_batch_aggregates_touched_surfaces_and_remains_inspectable() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.aspect-batch")
        .expect("task runtime should open a named workspace");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.batch-table", |q| {
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
                .schema_basis("tasks-batch-table")
        })
        .expect("live view should declare");
    let computed: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
        .computed(
            request_execution,
            "tasks.batch-summary",
            |c| {
                c.depends_on_live(&live)
                    .reads(test_aspect_touches(["title.value"]))
                    .produces(test_aspect_touches(["ui.batch_summary"]))
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
    let patches = workspace.observe(&live);
    let materialized = workspace
        .materialize_result(request_execution, &computed)
        .expect("computed materialization should execute");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("batch receipt should inspect");
    let state = workspace
        .state(request_execution, &receipt)
        .expect("batch receipt should expose state posture");

    assert_eq!(
        receipt.authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(
        receipt.basis_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
    assert_eq!(receipt.write_count(), 2);
    assert_eq!(receipt.considered_computed_view_count(), 1);
    assert_eq!(receipt.batch_mutation_evidence().component_count(), 2);
    assert_eq!(receipt.batch_mutation_evidence().target_evidence_count(), 2);
    assert_eq!(receipt.batch_mutation_evidence().resolved_target_count(), 2);
    assert_eq!(
        receipt.batch_mutation_evidence().target_collection_count(),
        0
    );
    assert_eq!(receipt.batch_mutation_evidence().target_entity_count(), 2);
    assert_eq!(
        receipt.batch_mutation_evidence().causality_bundle_count(),
        2
    );
    assert_eq!(
        receipt.batch_mutation_evidence().provenance_bundle_count(),
        2
    );
    assert_eq!(receipt.batch_mutation_evidence().outcome_class_count(), 2);
    assert_eq!(
        receipt.batch_mutation_evidence().authority_request_count(),
        2
    );
    assert_eq!(
        receipt.batch_mutation_evidence().authority_receipt_count(),
        2
    );
    assert!(!receipt
        .batch_mutation_evidence()
        .aggregate_target_digest()
        .is_empty());
    assert!(receipt
        .batch_mutation_evidence()
        .aggregate_causality_digest()
        .is_some());
    assert!(receipt
        .batch_mutation_evidence()
        .aggregate_provenance_digest()
        .is_some());
    assert_eq!(
        receipt.admitted_touched_aspects(),
        test_aspect_touches(["identity", "title"]).as_slice()
    );
    assert_eq!(
        receipt.terminal_affected_live_view_ids_projection(),
        &["tasks.batch-table".to_string()]
    );
    assert_eq!(
        receipt.terminal_affected_derived_view_ids_projection(),
        &["tasks.batch-summary".to_string()]
    );
    assert_eq!(patches.query_delivery_batches.len(), 2);
    assert_eq!(materialized.row_count(), 2);

    match inspection {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert_eq!(inspection.write_receipt_count(), 2);
            assert_eq!(
                inspection.admitted_touched_aspects(),
                test_aspect_touches(["identity", "title"]).as_slice()
            );
            assert_eq!(
                inspection.terminal_affected_live_view_ids_projection(),
                &["tasks.batch-table".to_string()]
            );
            assert_eq!(
                inspection.terminal_affected_derived_view_ids_projection(),
                &["tasks.batch-summary".to_string()]
            );
            assert_eq!(inspection.commit_identities().len(), 2);
            assert_eq!(inspection.component_operations().len(), 2);
            assert_eq!(inspection.component_operations()[0].family(), "insert");
            assert_eq!(
                inspection.component_operations()[0].declared_collection(),
                Some("Task")
            );
            assert_eq!(
                inspection.component_operations()[0].target_collection(),
                Some("Task")
            );
            assert_eq!(
                inspection.component_operations()[0]
                    .target_evidence()
                    .declared()
                    .target_class(),
                WorthQueryMutationTargetClass::Collection
            );
            assert!(inspection.component_operations()[0]
                .causality_evidence()
                .is_some());
            assert!(inspection.component_operations()[0]
                .provenance_evidence()
                .is_some());
            assert_eq!(
                inspection.component_operations()[0].collections(),
                &["Task".to_string()]
            );
            assert_eq!(
                inspection.component_operations()[0]
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
                vec!["set:identity:id".to_string(), "set:title:value".to_string()]
            );
            assert_eq!(
                inspection.basis_lane(),
                WorthQueryAuthorityLane::AuthoritativeTruth
            );
            assert_eq!(inspection.batch_mutation_evidence().component_count(), 2);
            assert_eq!(
                inspection.batch_mutation_evidence().target_evidence_count(),
                2
            );
            assert_eq!(
                inspection.batch_mutation_evidence().resolved_target_count(),
                2
            );
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .target_collection_count(),
                0
            );
            assert_eq!(
                inspection.batch_mutation_evidence().target_entity_count(),
                2
            );
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .causality_bundle_count(),
                2
            );
            assert_eq!(
                inspection
                    .batch_mutation_evidence()
                    .provenance_bundle_count(),
                2
            );
            assert_eq!(
                inspection.batch_mutation_evidence().outcome_class_count(),
                2
            );
            assert!(inspection
                .batch_mutation_evidence()
                .aggregate_causality_digest()
                .is_some());
            assert!(inspection
                .batch_mutation_evidence()
                .aggregate_provenance_digest()
                .is_some());
            assert!(!inspection.batch_digest().is_empty());
            assert!(!inspection.inspection_digest().is_empty());
        }
        other => panic!("expected batch receipt inspection, got {other:?}"),
    }
    assert_eq!(state.kind(), WorthQueryRuntimeStateKind::Ready);
    assert_eq!(
        state.authority_lane(),
        WorthQueryAuthorityLane::AuthoritativeTruth
    );
}

#[test]
fn preview_batch_stages_multiple_aspect_native_writes_in_preview_lane() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.preview-batch")
        .expect("task runtime should open a named workspace");
    let mut preview = workspace
        .preview_with_options(
            request_execution,
            test_session_label("task-preview-batch"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("preview should open");

    let receipt = preview
        .batch(request_execution, |batch| {
            batch
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("preview-task-1"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Preview title one"),
                    )
                })
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("preview-task-2"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Preview title two"),
                    )
                })
        })
        .expect("preview batch should stage");
    let outcome = preview.discard();

    assert_eq!(
        receipt.authority_lane(),
        WorthQueryAuthorityLane::PreviewTruth
    );
    assert_eq!(receipt.basis_lane(), WorthQueryAuthorityLane::PreviewTruth);
    assert_eq!(receipt.write_count(), 2);
    assert_eq!(receipt.considered_computed_view_count(), 0);
    assert_eq!(
        receipt.admitted_touched_aspects(),
        test_aspect_touches(["identity.id", "title.value"]).as_slice()
    );
    assert_eq!(outcome.authoritative_residue_count(), 0);
}

#[test]
fn preview_batch_uses_batch_target_evidence_without_authority_bundles() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.preview-batch-evidence")
        .expect("task runtime should open a named workspace");
    let mut preview = workspace
        .preview_with_options(
            request_execution,
            test_session_label("task-preview-batch-evidence"),
            WorthQueryPreviewOptions::sandboxed_write_intent(),
        )
        .expect("preview should open");

    let receipt = preview
        .batch(request_execution, |batch| {
            batch
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("preview-task-1"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Preview title one"),
                    )
                })
                .insert("Task", |task| {
                    task.set_aspect(
                        test_aspect_touch("identity.id"),
                        test_authored_string_aspect_value("preview-task-2"),
                    )
                    .set_aspect(
                        test_aspect_touch("title.value"),
                        test_authored_string_aspect_value("Preview title two"),
                    )
                })
        })
        .expect("preview batch should stage");

    assert_eq!(receipt.batch_mutation_evidence().component_count(), 2);
    assert_eq!(receipt.batch_mutation_evidence().target_evidence_count(), 2);
    assert_eq!(receipt.batch_mutation_evidence().resolved_target_count(), 0);
    assert_eq!(
        receipt.batch_mutation_evidence().target_collection_count(),
        2
    );
    assert_eq!(receipt.batch_mutation_evidence().target_entity_count(), 0);
    assert_eq!(
        receipt.batch_mutation_evidence().causality_bundle_count(),
        0
    );
    assert_eq!(
        receipt.batch_mutation_evidence().provenance_bundle_count(),
        0
    );
    assert!(receipt
        .batch_mutation_evidence()
        .aggregate_causality_digest()
        .is_none());
    assert!(receipt
        .batch_mutation_evidence()
        .aggregate_provenance_digest()
        .is_none());
}
