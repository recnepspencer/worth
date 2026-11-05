use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_epoch_fixture;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::WORKFLOW_STAGE;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_epoch_fixture;
use worth_query_execution::court_doors_for_test::domain_computation::WorthQueryConverged;
use worth_query_execution::court_doors_for_test::domain_computation::WorthQueryIndeterminate;
use worth_query_execution::court_doors_for_test::domain_computation::WorthQueryStableWithoutProof;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceTerminal;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceTerminal;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;

pub fn direct_terminal_outcome(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    disposition: FixtureDisposition,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceIterationOutcome {
    let epoch = direct_epoch_fixture(disposition, resource_request);
    let started = {
        let fixture_phase_result = {
            let fixture_target = epoch;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
            fixture_target.begin_iteration(
                &installed_fixture_phase,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "terminal-matrix",
                ),
            )
        };
        match fixture_phase_result {
            Ok(started) => started,
            Err(_) => panic!("terminal matrix iteration must start"),
        }
    };
    {
        let fixture_phase_result = {
            let fixture_target = started;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
            fixture_target.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
            _ => panic!("terminal matrix provider must complete and rejoin"),
        }
    }
}

pub fn converged_terminal(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryConverged> {
    match direct_terminal_outcome(execution, FixtureDisposition::Converged, resource_request) {
        WorthQueryDirectConvergenceIterationOutcome::Converged(terminal) => terminal,
        _ => panic!("converged fixture reached the wrong terminal"),
    }
}

pub fn stable_without_proof_terminal(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryStableWithoutProof> {
    match direct_terminal_outcome(
        execution,
        FixtureDisposition::StableWithoutProof,
        resource_request,
    ) {
        WorthQueryDirectConvergenceIterationOutcome::StableWithoutProof(terminal) => terminal,
        _ => panic!("stable-without-proof fixture reached the wrong terminal"),
    }
}

pub fn indeterminate_terminal(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    disposition: FixtureDisposition,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryIndeterminate> {
    match direct_terminal_outcome(execution, disposition, resource_request) {
        WorthQueryDirectConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
        _ => panic!("indeterminate fixture reached the wrong terminal"),
    }
}

pub fn workflow_converged_terminal(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWorkflowConvergenceTerminal<WorthQueryConverged> {
    match workflow_terminal_outcome(execution, FixtureDisposition::Converged, resource_request) {
        WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal) => terminal,
        _ => panic!("ordinary workflow comparator must converge"),
    }
}

pub fn workflow_indeterminate_terminal(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    disposition: FixtureDisposition,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWorkflowConvergenceTerminal<WorthQueryIndeterminate> {
    match workflow_terminal_outcome(execution, disposition, resource_request) {
        WorthQueryWorkflowConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
        _ => panic!("ordinary workflow comparator must remain indeterminate"),
    }
}

pub fn workflow_terminal_outcome(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    disposition: FixtureDisposition,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryWorkflowConvergenceIterationOutcome {
    let epoch = workflow_epoch_fixture(disposition, resource_request);
    let started = {
        let fixture_phase_result = {
            let fixture_target = epoch;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
            fixture_target.begin_stage_iteration(
                &installed_fixture_phase,
                WORKFLOW_STAGE,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "ordinary-workflow-convergence-iteration",
                ),
            )
        };
        match fixture_phase_result {
            Ok(started) => started,
            Err(_) => panic!("ordinary workflow iteration must start"),
        }
    };
    {
        let fixture_phase_result = {
            let fixture_target = started;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
            fixture_target.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceStepOutcome::Completed(outcome) => outcome,
            _ => panic!("ordinary workflow provider must complete and rejoin"),
        }
    }
}
