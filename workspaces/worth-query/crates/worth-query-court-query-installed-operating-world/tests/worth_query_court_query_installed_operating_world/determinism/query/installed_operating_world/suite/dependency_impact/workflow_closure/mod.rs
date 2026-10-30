pub(crate) use worth_query_testkit_query::facade::query::suite::dependency_impact::workflow_closure::*;

#[test]
fn certification_replay_preserves_the_compiled_workflow_closure() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workflow_workspace(request_execution, "dependency-impact-replay").unwrap();
    let original = bind_workflow(&workspace)
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();
    let original_closure = original.semantic_aspect_dependency_closure().unwrap();

    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        bind_workflow(&workspace),
        intent(),
        worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    assert_exact_d_work(original_closure, original_closure.dependencies().len());
}
