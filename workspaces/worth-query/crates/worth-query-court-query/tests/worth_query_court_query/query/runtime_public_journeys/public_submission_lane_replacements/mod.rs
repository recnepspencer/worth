pub(crate) use worth_query_testkit_query::facade::query::runtime_public_journeys::public_submission_lane_replacements::*;

#[test]
fn public_submission_lane_submit_replaces_direct_workspace_write() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("public.submission-lane.scalar")
        .expect("runtime should open a public workspace");
    let tasks = task_live_view(
        request_execution,
        &mut workspace,
        "public-submission-lane-scalar-tasks",
    );

    let receipt = workspace
        .submissions()
        .expect("submission lane should mint")
        .submit(
            request_execution,
            task_insert_command("task-submit-1", "Submitted scalar task"),
        )
        .expect("submission lane scalar write should execute");

    assert_eq!(receipt.mutation_family(), WorthQueryMutationFamily::Insert);
    assert_eq!(
        receipt.terminal_declared_collection_projection(),
        Some("Task")
    );

    let rows = workspace.read(request_execution, &tasks);
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0].scalar_value_at(&field_path("identity.id")),
        Some(&text("task-submit-1"))
    );
    assert_eq!(
        rows[0].scalar_value_at(&field_path("title.value")),
        Some(&text("Submitted scalar task"))
    );
}

#[test]
fn public_submission_lane_submit_batch_replaces_direct_workspace_batch() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let harness = PublicBridgeRuntimeHarness::new();
    let runtime = harness.bridge_backed_runtime(request_execution);
    let mut workspace = runtime
        .workspace("public.submission-lane.batch")
        .expect("runtime should open a public workspace");
    let tasks = task_live_view(
        request_execution,
        &mut workspace,
        "public-submission-lane-batch-tasks",
    );

    let receipt = workspace
        .submissions()
        .expect("submission lane should mint")
        .submit_batch(
            request_execution,
            vec![
                task_insert_command("task-batch-1", "Submitted batch one"),
                task_insert_command("task-batch-2", "Submitted batch two"),
            ],
        )
        .expect("submission lane batch write should execute");

    assert_eq!(receipt.write_count(), 2);
    assert!(receipt
        .write_receipts()
        .iter()
        .all(|write| write.mutation_family() == WorthQueryMutationFamily::Insert));

    let rows = workspace.read(request_execution, &tasks);
    assert_eq!(rows.len(), 2);
    let mut titles = rows
        .iter()
        .map(|row| {
            scalar_text(row.scalar_value_at(&field_path("title.value")))
                .expect("title should materialize")
                .to_string()
        })
        .collect::<Vec<_>>();
    titles.sort_unstable();

    assert_eq!(titles, vec!["Submitted batch one", "Submitted batch two"]);
}
