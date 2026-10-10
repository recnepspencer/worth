use super::super::support::*;

#[test]
fn batch_write_retained_artifact_keeps_receipt_inspection_and_exact_retained_binding_together() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("tasks.post-write-artifact")
        .expect("task runtime should open a named workspace");
    let live: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(request_execution, "tasks.post-write-table", |q| {
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
                .schema_basis("tasks-post-write-table")
        })
        .expect("live view should declare");
    let computed: WorthQueryDerivedViewHandle<WorthQueryUnrefinedLiveShape> = workspace
        .computed(
            request_execution,
            "tasks.post-write-summary",
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
            batch.insert("Task", |task| {
                task.set_aspect(
                    test_aspect_touch("identity.id"),
                    test_authored_string_aspect_value("task-1"),
                )
                .set_aspect(
                    test_aspect_touch("title.value"),
                    test_authored_string_aspect_value("Buy milk"),
                )
            })
        })
        .expect("batch should execute");

    let artifact = workspace
        .materialize_batch_write_artifact_binding(
            request_execution,
            &receipt,
            "tasks.post-write-artifact",
            [(&computed).into()],
        )
        .expect("post-write retained artifact should build");
    let inspection = workspace
        .inspect(request_execution, &receipt)
        .expect("receipt should inspect");
    let expected_row = workspace
        .materialize_result(request_execution, &computed)
        .expect("computed materialization should execute")
        .single_retained_row()
        .expect("computed materialization should retain one row")
        .clone();
    let retained_row = artifact
        .retained_artifact()
        .materialization(&computed)
        .expect("retained artifact should carry computed materialization")
        .single_retained_row()
        .expect("retained artifact should carry one retained row")
        .clone();

    assert_eq!(artifact.receipt().batch_digest(), receipt.batch_digest());
    match inspection {
        WorthQueryInspection::BatchWriteReceipt(inspection) => {
            assert_eq!(
                artifact.inspection().inspection_digest(),
                inspection.inspection_digest()
            );
        }
        other => panic!("expected batch-write receipt inspection, got {other:?}"),
    }
    assert_eq!(
        artifact.retained_artifact().artifact_name(),
        "tasks.post-write-artifact"
    );
    assert_eq!(artifact.retained_artifact().target_count(), 1);
    assert_eq!(
        artifact
            .retained_artifact()
            .terminal_target_view_names_projection()
            .collect::<Vec<_>>(),
        vec!["tasks.post-write-summary"]
    );
    assert_eq!(retained_row, expected_row);
}
