use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_admission_fixture_with_report_history_probe;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::WORKFLOW_STAGE;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::report_history_probe::FixtureReportHistoryObservation;
use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_admission_fixture_with_report_history_probe;
use worth_query_execution::court_doors_for_test::domain_computation::WorthQueryConverged;
use worth_query_execution::facade::convergence_epoch::WorthQueryBoundConvergenceReport;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceTerminal;
use worth_query_execution::facade::convergence_epoch::WorthQueryRetainedConvergenceCandidateEvidence;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceCleanupOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceTerminal;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;

#[test]
fn same_semantic_direct_peers_bind_distinct_report_histories() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (left_fixture, left_probe) = direct_admission_fixture_with_report_history_probe(
            FixtureDisposition::Converged,
            resource_request,
        );
        let (right_fixture, right_probe) = direct_admission_fixture_with_report_history_probe(
            FixtureDisposition::Converged,
            resource_request,
        );
        let left_started = {
            let fixture_state = left_fixture.admit();
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.begin_iteration(&installed_fixture_phase, request("same-report-scope"))
        }
        .unwrap_or_else(|_| panic!("left direct peer must begin"));
        let right_started = {
            let fixture_state = right_fixture.admit();
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.begin_iteration(&installed_fixture_phase, request("same-report-scope"))
        }
        .unwrap_or_else(|_| panic!("right direct peer must begin"));
        let left = direct_terminal({
            let installed_fixture_phase = left_started.fixture_phase_for_test(execution);
            left_started.advance(&installed_fixture_phase)
        });
        let right = direct_terminal({
            let installed_fixture_phase = right_started.fixture_phase_for_test(execution);
            right_started.advance(&installed_fixture_phase)
        });

        assert_distinct_histories(
            left.latest_report().unwrap(),
            left.incumbents(),
            &left_probe.observations()[0],
            right.latest_report().unwrap(),
            right.incumbents(),
            &right_probe.observations()[0],
        );
        assert!(left.cleanup().is_ok());
        assert!(right.cleanup().is_ok());
    });
}

#[test]
fn same_stage_workflow_peers_bind_distinct_report_histories() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (left_fixture, left_probe) = workflow_admission_fixture_with_report_history_probe(
            FixtureDisposition::Converged,
            resource_request,
        );
        let (right_fixture, right_probe) = workflow_admission_fixture_with_report_history_probe(
            FixtureDisposition::Converged,
            resource_request,
        );
        let left_started = {
            let fixture_state = left_fixture.admit();
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.begin_stage_iteration(
                &installed_fixture_phase,
                WORKFLOW_STAGE,
                request("same-workflow-report-scope"),
            )
        }
        .unwrap_or_else(|_| panic!("left workflow peer must begin"));
        let right_started = {
            let fixture_state = right_fixture.admit();
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.begin_stage_iteration(
                &installed_fixture_phase,
                WORKFLOW_STAGE,
                request("same-workflow-report-scope"),
            )
        }
        .unwrap_or_else(|_| panic!("right workflow peer must begin"));
        let left = workflow_terminal({
            let installed_fixture_phase = left_started.fixture_phase_for_test(execution);
            left_started.advance(&installed_fixture_phase)
        });
        let right = workflow_terminal({
            let installed_fixture_phase = right_started.fixture_phase_for_test(execution);
            right_started.advance(&installed_fixture_phase)
        });

        assert_distinct_histories(
            left.latest_report().unwrap(),
            left.incumbents(),
            &left_probe.observations()[0],
            right.latest_report().unwrap(),
            right.incumbents(),
            &right_probe.observations()[0],
        );
        assert!(matches!(
            left.cleanup(),
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(_)
        ));
        assert!(matches!(
            right.cleanup(),
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(_)
        ));
    });
}

pub fn assert_distinct_histories(
    left_report: &WorthQueryBoundConvergenceReport,
    left_incumbents: &[WorthQueryRetainedConvergenceCandidateEvidence],
    left_observation: &FixtureReportHistoryObservation,
    right_report: &WorthQueryBoundConvergenceReport,
    right_incumbents: &[WorthQueryRetainedConvergenceCandidateEvidence],
    right_observation: &FixtureReportHistoryObservation,
) {
    assert_eq!(
        left_report.decision().candidate_selection_key(),
        right_report.decision().candidate_selection_key()
    );
    assert_eq!(
        left_report.decision().state_identity(),
        right_report.decision().state_identity()
    );
    assert_ne!(
        left_report.evidence_identity(),
        right_report.evidence_identity()
    );
    assert_ne!(
        left_incumbents[0].occurrence_identity(),
        right_incumbents[0].occurrence_identity()
    );
    assert_eq!(
        left_incumbents[0].report_evidence_identity(),
        left_report.evidence_identity()
    );
    assert_eq!(
        right_incumbents[0].report_evidence_identity(),
        right_report.evidence_identity()
    );
    assert_report_origin(left_report, left_observation);
    assert_report_origin(right_report, right_observation);
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
}

pub fn request(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}

pub fn direct_terminal(
    outcome: WorthQueryDirectConvergenceStepOutcome,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryConverged> {
    match outcome {
        WorthQueryDirectConvergenceStepOutcome::Completed(
            WorthQueryDirectConvergenceIterationOutcome::Converged(terminal),
        ) => terminal,
        _ => panic!("direct peer must converge"),
    }
}

pub fn workflow_terminal(
    outcome: WorthQueryWorkflowConvergenceStepOutcome,
) -> WorthQueryWorkflowConvergenceTerminal<WorthQueryConverged> {
    match outcome {
        WorthQueryWorkflowConvergenceStepOutcome::Completed(
            WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal),
        ) => terminal,
        _ => panic!("workflow peer must converge"),
    }
}
