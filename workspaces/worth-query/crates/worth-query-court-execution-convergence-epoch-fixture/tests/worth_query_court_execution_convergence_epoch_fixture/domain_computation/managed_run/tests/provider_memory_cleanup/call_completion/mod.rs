use crate::domain_computation::managed_run::tests::provider_memory_cleanup::*;

pub fn complete_call(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    running: WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    scope: &str,
) -> WorthQueryRunningDirectRun {
    let active = {
        let installed_fixture_phase = running.fixture_phase_for_test(execution);
        running.begin_graph_execution(
            &installed_fixture_phase,
            graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
    }
    .expect("provider call should start");
    {
        let fixture_phase_result = {
            let installed_fixture_phase = active.fixture_phase_for_test(execution);
            active.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion.into_running(),
            _ => panic!("escaping-memory provider did not complete"),
        }
    }
}

pub fn complete_workflow_memory_call(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    running: worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    scope: &str,
) -> worth_query_execution::facade::domain_computation::WorthQueryRunningWorkflowRun {
    let active = {
        let installed_fixture_phase = running.fixture_phase_for_test(execution);
        running.begin_stage_graph_execution(
            &installed_fixture_phase,
            "stage",
            graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
    }
    .expect("workflow memory provider call should start");
    {
        let fixture_phase_result = {
            let installed_fixture_phase = active.fixture_phase_for_test(execution);
            active.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion.into_running(),
            _ => panic!("workflow memory provider did not complete"),
        }
    }
}
