use crate::domain_computation::convergence_epoch::tests::yield_cleanup::*;

pub fn workflow_yield_with_pending_artifact(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    call_identity: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> (
    WorthQueryYieldedWorkflowConvergenceIteration,
    WorthQueryMoveOnlyArtifactHandle,
) {
    let (fixture, artifact_receiver) = workflow_yield_pending_admission_fixture(resource_request);
    let WorkflowAdmissionFixture {
        runtime,
        operation,
        contract,
        managed,
        graph,
        bridge: _,
    } = fixture;
    let admitted = runtime
        .admit_workflow_convergence_epoch(&operation, contract, managed, graph)
        .unwrap_or_else(|_| panic!("artifact workflow authorities must admit"));
    let epoch = admitted
        .start()
        .unwrap_or_else(|_| panic!("artifact workflow convergence must start"));
    let started = {
        let installed_fixture_phase = epoch.fixture_phase_for_test(execution);
        epoch.begin_stage_iteration(
            &installed_fixture_phase,
            WORKFLOW_STAGE,
            call(call_identity),
        )
    }
    .unwrap_or_else(|_| panic!("artifact workflow iteration must start"));
    let paused = {
        let fixture_phase_result = {
            let installed_fixture_phase = started.fixture_phase_for_test(execution);
            started.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceStepOutcome::Continue(paused) => paused,
            _ => panic!("artifact workflow must reach a yield safe point"),
        }
    };
    let artifact = artifact_receiver
        .recv()
        .expect("production provider step must issue the move-only artifact handle");
    let yielded = match paused.yield_iteration() {
        WorthQueryWorkflowConvergenceYieldOutcome::Yielded(yielded) => yielded,
        _ => panic!("artifact workflow must yield"),
    };
    (yielded, artifact)
}
