pub(crate) use worth_query_testkit_query::facade::query::suite::workflow_parallel_progression::*;

#[test]
fn admitted_parallel_frontier_retains_lower_proof_and_converges_with_serial_trace() {
    let host_request = worth_query_testkit_installed_operation_fixture::facade::query::test_execution_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let serial = complete_serial_trace(request_execution, "workflow-serial-reference");
    let mut workspace =
        workflow_workspace(request_execution, "workflow-parallel-frontier").unwrap();
    let installed_domain = workspace.domain(GeometryDomain).unwrap();
    let run = workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed_domain, WorkflowRead)
        .unwrap()
        .admit_workflow_resources(
            worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .start_workflow(&mut workspace)
        .unwrap()
        .advance(
            "start",
            domain::WorthQueryWorkflowValue::NotRequired,
            &mut workspace,
        )
        .unwrap()
        .advance_admitted_frontier(
            [
                (
                    "right".into(),
                    domain::WorthQueryWorkflowValue::Text("start".into()),
                ),
                (
                    "left".into(),
                    domain::WorthQueryWorkflowValue::Text("start".into()),
                ),
            ],
            &mut workspace,
        )
        .unwrap()
        .advance(
            "publish",
            domain::WorthQueryWorkflowValue::Text("join".into()),
            &mut workspace,
        )
        .unwrap();
    let trace = run.complete().unwrap();

    assert_eq!(trace.semantic_identity(), serial);
    assert_eq!(trace.counters().parallel_admission_checks, 1);
    for stage in trace
        .stage_receipts()
        .iter()
        .filter(|receipt| matches!(receipt.stage_identity(), "left" | "right"))
    {
        let proof = stage.parallel_admission().unwrap();
        assert_eq!(proof.run_identity(), stage.run_identity());
        assert_eq!(
            proof
                .frontier()
                .iter()
                .map(|stage| stage.stage_identity())
                .collect::<Vec<_>>(),
            ["left", "right"]
        );
        assert!(proof.lower_receipt().is_parallel_admitted());
        assert_eq!(stage.counters().parallel_admission_checks, 0);
    }
}
