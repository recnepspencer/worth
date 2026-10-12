use super::support::*;

#[test]
fn ordinary_runtime_backed_read_path_keeps_forbidden_fallback_seams_at_exact_zero() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let runtime = stateful_bridge_task_runtime(request_execution);
    let mut workspace = runtime
        .workspace("runtime.tests.hostile-read-bootstrap")
        .expect("task runtime should open a named workspace");
    let tasks: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view(
            request_execution,
            "runtime.tests.hostile-read-bootstrap.tasks",
            |q| {
                q.from("Task")
                    .select([identity_id_field_key(), title_value_field_key()])
                    .order_by(title_value_field_key())
                    .schema_basis("runtime-tests-hostile-read-bootstrap-tasks")
            },
        )
        .expect("task live view should declare");

    reset_forbidden_fallback_seam_invocations();

    workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-hostile-bootstrap"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Hostile bootstrap task"),
            )
        })
        .expect("insert should execute through the ordinary bridge-backed lane");

    let rows = workspace.read(request_execution, &tasks);
    let patches = workspace.observe(&tasks);

    assert_eq!(rows.len(), 1);
    assert_eq!(patches.query_delivery_batches.len(), 1);
    for seam in [
        WorthQueryForbiddenFallbackSeam::ConsumeScalarFields,
        WorthQueryForbiddenFallbackSeam::DecodeRowPair,
        WorthQueryForbiddenFallbackSeam::DecodeRowTriple,
        WorthQueryForbiddenFallbackSeam::VerifyScalarAlignment,
        WorthQueryForbiddenFallbackSeam::ReadLiveArtifactBundle,
        WorthQueryForbiddenFallbackSeam::BindLiveArtifact,
        WorthQueryForbiddenFallbackSeam::ReadLiveArtifactBinding,
    ] {
        assert_eq!(forbidden_fallback_seam_invocation_count(seam), 0);
    }
}
