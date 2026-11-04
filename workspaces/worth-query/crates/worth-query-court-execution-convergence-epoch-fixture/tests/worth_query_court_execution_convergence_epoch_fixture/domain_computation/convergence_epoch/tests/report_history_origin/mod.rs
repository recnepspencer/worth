use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_admission_fixture_with_report_history_probe;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::WORKFLOW_STAGE;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::report_history_probe::FixtureReportHistoryObservation;
use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_admission_fixture_with_report_history_probe;
use worth_query_execution::facade::convergence_epoch::WorthQueryBoundConvergenceReport;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceCleanupOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceStepOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;

#[test]
fn direct_report_origin_matches_the_installed_provider_observation() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (fixture, probe) = direct_admission_fixture_with_report_history_probe(
            FixtureDisposition::Converged,
            resource_request,
        );
        let started = {
            let fixture_state = fixture.admit();
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.begin_iteration(&installed_fixture_phase, request("direct-report-origin"))
        }
        .unwrap_or_else(|_| panic!("direct report origin iteration must start"));
        let terminal = {
            let fixture_phase_result = {
                let installed_fixture_phase = started.fixture_phase_for_test(execution);
                started.advance(&installed_fixture_phase)
            };
            match fixture_phase_result {
                WorthQueryDirectConvergenceStepOutcome::Completed(
                    WorthQueryDirectConvergenceIterationOutcome::Converged(terminal),
                ) => terminal,
                _ => panic!("direct report origin fixture must converge"),
            }
        };
        assert_report_origin(
            terminal
                .latest_report()
                .expect("converged direct epoch must retain its report"),
            single_observation(&probe.observations()),
        );
        assert!(terminal.cleanup().is_ok());
    });
}

#[test]
fn workflow_report_origin_matches_the_installed_provider_observation() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (fixture, probe) = workflow_admission_fixture_with_report_history_probe(
            FixtureDisposition::Converged,
            resource_request,
        );
        let started = {
            let fixture_state = fixture.admit();
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.begin_stage_iteration(
                &installed_fixture_phase,
                WORKFLOW_STAGE,
                request("workflow-report-origin"),
            )
        }
        .unwrap_or_else(|_| panic!("workflow report origin iteration must start"));
        let terminal = {
            let fixture_phase_result = {
                let installed_fixture_phase = started.fixture_phase_for_test(execution);
                started.advance(&installed_fixture_phase)
            };
            match fixture_phase_result {
                WorthQueryWorkflowConvergenceStepOutcome::Completed(
                    WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal),
                ) => terminal,
                _ => panic!("workflow report origin fixture must converge"),
            }
        };
        assert_report_origin(
            terminal
                .latest_report()
                .expect("converged workflow epoch must retain its report"),
            single_observation(&probe.observations()),
        );
        assert!(matches!(
            terminal.cleanup(),
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(_)
        ));
    });
}

pub fn request(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}

pub fn single_observation(
    observations: &[FixtureReportHistoryObservation],
) -> &FixtureReportHistoryObservation {
    assert_eq!(observations.len(), 1);
    &observations[0]
}

pub fn assert_report_origin(
    report: &WorthQueryBoundConvergenceReport,
    observation: &FixtureReportHistoryObservation,
) {
    assert_eq!(report.iteration_ordinal(), observation.iteration_ordinal());
    assert_eq!(
        report.provider_receipt_identity(),
        observation.provider_receipt_identity()
    );
    assert_eq!(
        report.graph_evidence_identity(),
        observation.graph_evidence_identity()
    );
    assert!(observation.incumbents().is_empty());
}
