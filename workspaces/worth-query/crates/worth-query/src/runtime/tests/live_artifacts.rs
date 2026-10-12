use super::support::*;

#[test]
fn workspace_reads_live_artifact_binding_as_one_snapshot_coherent_named_pack() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = stateful_bridge_task_runtime(request_execution)
        .workspace("live.artifact.binding")
        .expect("task runtime should open a named workspace");
    let entities: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view_request(
            request_execution,
            "tasks.live-entities",
            task_live_request(),
            task_schema(),
        )
        .expect("entity live view should declare");
    let names: WorthQueryLiveView<WorthQueryUnrefinedLiveShape> = workspace
        .live_view_request(
            request_execution,
            "tasks.live-names",
            task_live_request(),
            task_schema(),
        )
        .expect("naming live view should declare");

    workspace
        .insert(request_execution, "Task", |task| {
            task.set_aspect(
                test_aspect_touch("identity.id"),
                test_authored_string_aspect_value("task-1"),
            )
            .set_aspect(
                test_aspect_touch("title.value"),
                test_authored_string_aspect_value("Live artifact"),
            )
        })
        .expect("workspace write should execute");

    let binding = workspace
        .read_live_artifact_binding(
            request_execution,
            "tasks.historical.naming-pack",
            [(&entities).into(), (&names).into()],
        )
        .expect("live artifact binding should materialize");

    assert_eq!(binding.artifact_name(), "tasks.historical.naming-pack");
    assert_eq!(binding.target_count(), 2);
    assert_eq!(
        binding.read(&entities).expect("entity rows").rows().len(),
        1
    );
    assert_eq!(binding.read(&names).expect("name rows").rows().len(), 1);
    assert_eq!(
        binding
            .read(&entities)
            .expect("entity rows")
            .receipt()
            .snapshot_identity(),
        binding
            .read(&names)
            .expect("name rows")
            .receipt()
            .snapshot_identity(),
    );
    assert_eq!(
        binding.snapshot_identity(),
        binding
            .read(&entities)
            .expect("entity rows")
            .receipt()
            .snapshot_identity(),
    );
}
