use super::fixture::{
    direct_admission_fixture_with_report_history_probe,
    workflow_admission_fixture_with_report_history_probe, FixtureDisposition,
    FixtureReportHistoryObservation, WORKFLOW_STAGE,
};
use crate::domain_computation::{
    WorthQueryBoundConvergenceReport, WorthQueryDirectConvergenceIterationOutcome,
    WorthQueryDirectConvergenceStepOutcome, WorthQueryGraphProviderCallKind,
    WorthQueryManagedGraphCallRequest, WorthQueryWorkflowConvergenceCleanupOutcome,
    WorthQueryWorkflowConvergenceIterationOutcome, WorthQueryWorkflowConvergenceStepOutcome,
};

#[test]
fn direct_report_origin_matches_the_installed_provider_observation() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (fixture, probe) =
            direct_admission_fixture_with_report_history_probe(FixtureDisposition::Converged);
        let started = fixture
            .admit()
            .begin_iteration(execution, request("direct-report-origin"))
            .unwrap_or_else(|_| panic!("direct report origin iteration must start"));
        let terminal = match started.advance(execution) {
            WorthQueryDirectConvergenceStepOutcome::Completed(
                WorthQueryDirectConvergenceIterationOutcome::Converged(terminal),
            ) => terminal,
            _ => panic!("direct report origin fixture must converge"),
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
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (fixture, probe) =
            workflow_admission_fixture_with_report_history_probe(FixtureDisposition::Converged);
        let started = fixture
            .admit()
            .begin_stage_iteration(execution, WORKFLOW_STAGE, request("workflow-report-origin"))
            .unwrap_or_else(|_| panic!("workflow report origin iteration must start"));
        let terminal = match started.advance(execution) {
            WorthQueryWorkflowConvergenceStepOutcome::Completed(
                WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal),
            ) => terminal,
            _ => panic!("workflow report origin fixture must converge"),
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

fn request(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}

fn single_observation(
    observations: &[FixtureReportHistoryObservation],
) -> &FixtureReportHistoryObservation {
    assert_eq!(observations.len(), 1);
    &observations[0]
}

fn assert_report_origin(
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
