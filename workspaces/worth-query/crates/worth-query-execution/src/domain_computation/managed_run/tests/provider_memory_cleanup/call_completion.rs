use super::*;

pub(super) fn complete_call(
    execution: &crate::domain_computation::primary_graph::TestHostRequest<'_>,

    running: WorthQueryRunningDirectRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    scope: &str,
) -> WorthQueryRunningDirectRun {
    let active = {
        let installed_fixture_phase = running.fixture_phase(execution);
        running.begin_graph_execution(
            &installed_fixture_phase,
            graph,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
    }
    .expect("provider call should start");
    {
        let fixture_phase_result = {
            let installed_fixture_phase = active.fixture_phase(execution);
            active.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion.into_running(),
            _ => panic!("escaping-memory provider did not complete"),
        }
    }
}

pub(super) fn complete_workflow_memory_call(
    execution: &crate::domain_computation::primary_graph::TestHostRequest<'_>,

    running: crate::domain_computation::WorthQueryRunningWorkflowRun,
    graph: &WorthQueryInstalledGraphParticipationAuthority,
    scope: &str,
) -> crate::domain_computation::WorthQueryRunningWorkflowRun {
    let active = {
        let installed_fixture_phase = running.fixture_phase(execution);
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
            let installed_fixture_phase = active.fixture_phase(execution);
            active.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowGraphStepOutcome::Completed(completion) => completion.into_running(),
            _ => panic!("workflow memory provider did not complete"),
        }
    }
}
