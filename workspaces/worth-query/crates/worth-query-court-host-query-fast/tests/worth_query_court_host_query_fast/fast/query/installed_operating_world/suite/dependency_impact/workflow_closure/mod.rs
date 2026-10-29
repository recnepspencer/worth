pub(crate) use worth_query_worlds::facade::query::suite::dependency_impact::workflow_closure::*;

#[test]
fn completed_workflow_closure_retains_declared_and_realized_roles_at_exact_d_cost() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace =
        mutation_workflow_workspace(request_execution, "dependency-impact-workflow").unwrap();
    let installed = workspace.domain(GeometryDomain).unwrap();
    let trace = workspace
        .prepare_mutation_operating_world(workspace.current_world())
        .unwrap()
        .family(MutationFamily)
        .bind(&installed, WorkflowMutation)
        .unwrap()
        .admit_workflow_resources(
            worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "mutate",
            domain::WorthQueryWorkflowValue::Text("commit".into()),
            &mut workspace,
        )
        .unwrap()
        .complete()
        .unwrap();
    let closure = trace.semantic_aspect_dependency_closure().unwrap();
    let d = closure.dependencies().len();

    assert_eq!(d, 16);
    assert_declared_mutation_dependencies(closure);
    assert_realized_mutation_dependencies(closure);
    assert_mutation_dependency_counters(closure);
    assert_exact_d_work(closure, d);
}

#[test]
fn certification_replay_preserves_the_compiled_workflow_closure() {
    let host_request = worth_query_worlds::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let mut workspace = workflow_workspace(request_execution, "dependency-impact-replay").unwrap();
    let original = bind_workflow(&workspace)
        .admit_workflow_resources(
            worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
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
        worth_query_worlds::facade::query::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    assert_eq!(
        replay.comparison(),
        &domain::WorthQueryReplayComparison::Equivalent
    );
    assert_exact_d_work(original_closure, original_closure.dependencies().len());
}
