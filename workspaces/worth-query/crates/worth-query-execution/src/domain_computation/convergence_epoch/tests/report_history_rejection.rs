use super::fixture::{
    direct_admission_fixture_with_report_history_probe,
    workflow_admission_fixture_with_report_history_probe, FixtureDisposition,
    FixtureReportHistoryObservation, WORKFLOW_STAGE,
};
use crate::domain_computation::{
    WorthQueryConvergenceEpochDenialKind, WorthQueryConvergenceIndeterminateCause,
    WorthQueryDirectConvergenceIterationOutcome, WorthQueryDirectConvergenceStepOutcome,
    WorthQueryGraphProviderCallKind, WorthQueryManagedGraphCallRequest,
    WorthQueryWorkflowConvergenceCleanupOutcome, WorthQueryWorkflowConvergenceIterationOutcome,
    WorthQueryWorkflowConvergenceStepOutcome,
};

#[test]
fn invalid_incumbent_transition_preserves_committed_direct_and_workflow_history() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (direct_fixture, direct_probe) = direct_admission_fixture_with_report_history_probe(
            FixtureDisposition::HistoryInvalidTransition,
            resource_request,
        );
        let direct_epoch =
            match advance_direct(execution, direct_fixture.admit(), "direct-history-valid") {
                WorthQueryDirectConvergenceIterationOutcome::Continue(epoch) => epoch,
                _ => panic!("first direct history report must commit"),
            };
        let direct = match advance_direct(execution, direct_epoch, "direct-history-invalid") {
            WorthQueryDirectConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
            _ => panic!("invalid direct incumbent transition must be indeterminate"),
        };
        assert_preserved_history(
            direct.latest_report().unwrap(),
            direct.incumbents(),
            direct_probe.observations(),
        );
        assert_denial(direct.indeterminate_cause());
        assert_eq!(direct.counters().iteration_count(), 2);
        assert_eq!(direct.counters().incumbent_replacement_count(), 1);
        assert_eq!(direct.counters().incumbent_retention_count(), 0);
        assert_eq!(direct.counters().comparator_call_count(), 2);
        assert_eq!(direct.counters().progress_check_count(), 2);
        assert_eq!(direct.counters().repeated_state_probe_count(), 2);
        assert!(direct.cleanup().is_ok());

        let (workflow_fixture, workflow_probe) =
            workflow_admission_fixture_with_report_history_probe(
                FixtureDisposition::HistoryInvalidTransition,
                resource_request,
            );
        let workflow_epoch = match advance_workflow(
            execution,
            workflow_fixture.admit(),
            "workflow-history-valid",
        ) {
            WorthQueryWorkflowConvergenceIterationOutcome::Continue(epoch) => epoch,
            _ => panic!("first workflow history report must commit"),
        };
        let workflow = match advance_workflow(execution, workflow_epoch, "workflow-history-invalid")
        {
            WorthQueryWorkflowConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
            _ => panic!("invalid workflow incumbent transition must be indeterminate"),
        };
        assert_preserved_history(
            workflow.latest_report().unwrap(),
            workflow.incumbents(),
            workflow_probe.observations(),
        );
        assert_denial(workflow.indeterminate_cause());
        assert_eq!(workflow.counters().iteration_count(), 2);
        assert_eq!(workflow.counters().incumbent_replacement_count(), 1);
        assert_eq!(workflow.counters().incumbent_retention_count(), 0);
        assert_eq!(workflow.counters().comparator_call_count(), 2);
        assert_eq!(workflow.counters().progress_check_count(), 2);
        assert_eq!(workflow.counters().repeated_state_probe_count(), 2);
        assert!(matches!(
            workflow.cleanup(),
            WorthQueryWorkflowConvergenceCleanupOutcome::Complete(_)
        ));
    });
}

#[test]
fn invalid_domain_report_preserves_the_previously_committed_history() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let (fixture, probe) = direct_admission_fixture_with_report_history_probe(
            FixtureDisposition::HistoryInvalidDomain,
            resource_request,
        );
        let epoch = match advance_direct(execution, fixture.admit(), "domain-history-valid") {
            WorthQueryDirectConvergenceIterationOutcome::Continue(epoch) => epoch,
            _ => panic!("first domain history report must commit"),
        };
        let terminal = match advance_direct(execution, epoch, "domain-history-invalid") {
            WorthQueryDirectConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
            _ => panic!("incoherent second domain report must be indeterminate"),
        };
        assert_preserved_history(
            terminal.latest_report().unwrap(),
            terminal.incumbents(),
            probe.observations(),
        );
        assert!(matches!(
            terminal.indeterminate_cause(),
            Some(WorthQueryConvergenceIndeterminateCause::ReportAdmission(denial))
                if denial.kind() == WorthQueryConvergenceEpochDenialKind::InvalidDomainReport
        ));
        assert_eq!(terminal.counters().iteration_count(), 2);
        assert_eq!(terminal.counters().incumbent_replacement_count(), 1);
        assert_eq!(terminal.counters().incumbent_retention_count(), 0);
        assert_eq!(terminal.counters().comparator_call_count(), 2);
        assert_eq!(terminal.counters().progress_check_count(), 2);
        assert_eq!(terminal.counters().repeated_state_probe_count(), 2);
        assert!(terminal.cleanup().is_ok());
    });
}

fn assert_preserved_history(
    report: &crate::domain_computation::WorthQueryBoundConvergenceReport,
    incumbents: &[crate::domain_computation::WorthQueryRetainedConvergenceCandidateEvidence],
    observations: Vec<FixtureReportHistoryObservation>,
) {
    assert_eq!(observations.len(), 2);
    let before_rejection = &observations[1];
    assert_eq!(before_rejection.incumbents().len(), 1);
    let prior = &before_rejection.incumbents()[0];
    assert_eq!(report.iteration_ordinal(), 1);
    assert_eq!(report.evidence_identity(), prior.report_evidence_identity());
    assert_eq!(incumbents.len(), 1);
    assert_eq!(
        incumbents[0].occurrence_identity(),
        prior.occurrence_identity()
    );
    assert_eq!(incumbents[0].state_identity(), prior.state_identity());
    assert_eq!(
        incumbents[0].report_evidence_identity(),
        prior.report_evidence_identity()
    );
}

fn assert_denial(cause: Option<&WorthQueryConvergenceIndeterminateCause>) {
    assert!(matches!(
        cause,
        Some(WorthQueryConvergenceIndeterminateCause::ReportAdmission(denial))
            if denial.kind() == WorthQueryConvergenceEpochDenialKind::InvalidIncumbentTransition
    ));
}

fn request(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}

fn advance_direct(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    epoch: crate::domain_computation::WorthQueryIteratingDirectConvergenceEpoch,
    identity: &str,
) -> WorthQueryDirectConvergenceIterationOutcome {
    let started = epoch
        .begin_iteration(execution, request(identity))
        .unwrap_or_else(|_| panic!("direct report history iteration must start"));
    match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("direct report history provider must complete"),
    }
}

fn advance_workflow(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    epoch: crate::domain_computation::WorthQueryIteratingWorkflowConvergenceEpoch,
    identity: &str,
) -> WorthQueryWorkflowConvergenceIterationOutcome {
    let started = epoch
        .begin_stage_iteration(execution, WORKFLOW_STAGE, request(identity))
        .unwrap_or_else(|_| panic!("workflow report history iteration must start"));
    match started.advance(execution) {
        WorthQueryWorkflowConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("workflow report history provider must complete"),
    }
}
