use super::support::*;

#[test]
fn redeclared_live_view_replaces_runtime_delivery_index_membership() {
    let mut runtime = test_product_runtime_builder()
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .write_authority(TestWriteAuthority)
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("native live redeclaration contracts should admit")
        .build_backend_from_parts()
        .build()
        .expect("bridge-backed runtime should build");
    let task_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("shared.surface", task_live_request(), task_schema())
        .expect("task live view should declare");
    let task_seed = runtime
        .write(insert_command(
            "Task",
            [
                ("identity.id", test_string_aspect_value("")),
                ("title.value", test_string_aspect_value("Task seed")),
            ],
        ))
        .expect("task seed should write");
    let _ = runtime.drain_patches(&task_view);

    let issue_view: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = runtime
        .declare_live_view("shared.surface", issue_live_request(), issue_schema())
        .expect("same live view name should redeclare against issue collection");
    let stale_task_update = runtime
        .write(test_update_string_aspect_command(
            task_seed.deltas()[0].entity_identity.clone(),
            "title.value",
            "Task update after redeclare",
        ))
        .expect("task update should still write");
    let stale_task_patches = runtime.drain_patches(&issue_view);

    assert!(stale_task_update
        .terminal_affected_live_view_ids_projection()
        .is_empty());
    assert!(stale_task_patches.query_delivery_batches.is_empty());

    let issue_write = runtime
        .write(insert_command(
            "Issue",
            [
                ("identity.id", test_string_aspect_value("")),
                ("summary.value", test_string_aspect_value("Issue seed")),
            ],
        ))
        .expect("issue insert should write");
    let issue_patches = runtime.drain_patches(&issue_view);

    assert_eq!(
        issue_write.terminal_affected_live_view_ids_projection(),
        &["shared.surface".to_string()]
    );
    assert_eq!(issue_patches.query_delivery_batches.len(), 1);
}
