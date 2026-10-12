use super::*;

#[test]
fn shared_read_contexts_at_same_generation_have_identical_basis_and_artifacts() {
    let host_request = crate::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let (mut workspace, derived) =
        shared_read_pinning_workspace(request_execution, "shared-read.phase13.boundary");

    let first = workspace
        .shared_read_context(request_execution)
        .expect("first context should mint");
    let second = workspace
        .shared_read_context(request_execution)
        .expect("second context should mint at same generation");
    let first_artifact = first
        .published_derived_artifact(&derived)
        .expect("first artifact should resolve");
    let second_artifact = second
        .published_derived_artifact(&derived)
        .expect("second artifact should resolve");

    assert_eq!(first.inspect_basis(), second.inspect_basis());
    assert_eq!(
        first.inspect_basis().snapshot_identity(),
        first_artifact
            .inspect_projection_consumption()
            .snapshot_identity()
    );
    assert_eq!(
        first_artifact
            .published_binding()
            .map(|binding| binding.binding_for_reporting()),
        second_artifact
            .published_binding()
            .map(|binding| binding.binding_for_reporting())
    );
    assert_eq!(
        consume_display_title(&first_artifact),
        consume_display_title(&second_artifact)
    );

    insert_task(request_execution, &mut workspace, "task-2", "Task Two");
    let newer = workspace
        .shared_read_context(request_execution)
        .expect("newer context should mint after commit");

    assert_ne!(first.inspect_basis(), newer.inspect_basis());
    assert_eq!(
        first_artifact
            .published_binding()
            .map(|binding| binding.binding_for_reporting()),
        first
            .published_derived_artifact(&derived)
            .expect("old context should remain on old published artifact")
            .published_binding()
            .map(|binding| binding.binding_for_reporting())
    );
    assert_eq!(
        consume_display_title(&first_artifact),
        consume_display_title(
            &first
                .published_derived_artifact(&derived)
                .expect("old context should continue consuming old facts")
        )
    );
}
