use super::fixture::{
    direct_admission_fixture_with_report_history_probe,
    workflow_admission_fixture_with_report_history_probe, FixtureDisposition,
    FixtureReportHistoryObservation, WORKFLOW_STAGE,
};
use crate::domain_computation::{
    WorthQueryBoundConvergenceReport, WorthQueryConverged,
    WorthQueryDirectConvergenceIterationOutcome, WorthQueryDirectConvergenceStepOutcome,
    WorthQueryDirectConvergenceTerminal, WorthQueryGraphProviderCallKind,
    WorthQueryManagedGraphCallRequest, WorthQueryRetainedConvergenceCandidateEvidence,
    WorthQueryWorkflowConvergenceCleanupOutcome, WorthQueryWorkflowConvergenceIterationOutcome,
    WorthQueryWorkflowConvergenceStepOutcome, WorthQueryWorkflowConvergenceTerminal,
};

#[test]
fn same_semantic_direct_peers_bind_distinct_report_histories() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (left_fixture, left_probe) =
            direct_admission_fixture_with_report_history_probe(FixtureDisposition::Converged);
        let (right_fixture, right_probe) =
            direct_admission_fixture_with_report_history_probe(FixtureDisposition::Converged);
        let left_started = left_fixture
            .admit()
            .begin_iteration(execution, request("same-report-scope"))
            .unwrap_or_else(|_| panic!("left direct peer must begin"));
        let right_started = right_fixture
            .admit()
            .begin_iteration(execution, request("same-report-scope"))
            .unwrap_or_else(|_| panic!("right direct peer must begin"));
        let left = direct_terminal(left_started.advance(execution));
        let right = direct_terminal(right_started.advance(execution));

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
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (left_fixture, left_probe) =
            workflow_admission_fixture_with_report_history_probe(FixtureDisposition::Converged);
        let (right_fixture, right_probe) =
            workflow_admission_fixture_with_report_history_probe(FixtureDisposition::Converged);
        let left_started = left_fixture
            .admit()
            .begin_stage_iteration(
                execution,
                WORKFLOW_STAGE,
                request("same-workflow-report-scope"),
            )
            .unwrap_or_else(|_| panic!("left workflow peer must begin"));
        let right_started = right_fixture
            .admit()
            .begin_stage_iteration(
                execution,
                WORKFLOW_STAGE,
                request("same-workflow-report-scope"),
            )
            .unwrap_or_else(|_| panic!("right workflow peer must begin"));
        let left = workflow_terminal(left_started.advance(execution));
        let right = workflow_terminal(right_started.advance(execution));

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

fn assert_distinct_histories(
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
}

fn request(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}

fn direct_terminal(
    outcome: WorthQueryDirectConvergenceStepOutcome,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryConverged> {
    match outcome {
        WorthQueryDirectConvergenceStepOutcome::Completed(
            WorthQueryDirectConvergenceIterationOutcome::Converged(terminal),
        ) => terminal,
        _ => panic!("direct peer must converge"),
    }
}

fn workflow_terminal(
    outcome: WorthQueryWorkflowConvergenceStepOutcome,
) -> WorthQueryWorkflowConvergenceTerminal<WorthQueryConverged> {
    match outcome {
        WorthQueryWorkflowConvergenceStepOutcome::Completed(
            WorthQueryWorkflowConvergenceIterationOutcome::Converged(terminal),
        ) => terminal,
        _ => panic!("workflow peer must converge"),
    }
}
