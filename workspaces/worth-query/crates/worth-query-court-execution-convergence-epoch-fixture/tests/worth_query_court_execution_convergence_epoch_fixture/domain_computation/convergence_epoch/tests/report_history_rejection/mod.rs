use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_admission_fixture_with_report_history_probe;
use crate::domain_computation::convergence_epoch::tests::fixture::fixture_identity::WORKFLOW_STAGE;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::report_history_probe::FixtureReportHistoryObservation;
use crate::domain_computation::convergence_epoch::tests::fixture::workflow_installed_world::workflow_admission_fixture_with_report_history_probe;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceEpochDenialKind;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceIndeterminateCause;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceCleanupOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryWorkflowConvergenceStepOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;

#[test]
fn invalid_incumbent_transition_preserves_committed_direct_and_workflow_history() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
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
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
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

pub fn assert_preserved_history(
    report: &worth_query_execution::facade::convergence_epoch::WorthQueryBoundConvergenceReport,
    incumbents: &[worth_query_execution::facade::convergence_epoch::WorthQueryRetainedConvergenceCandidateEvidence],
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

pub fn assert_denial(cause: Option<&WorthQueryConvergenceIndeterminateCause>) {
    assert!(matches!(
        cause,
        Some(WorthQueryConvergenceIndeterminateCause::ReportAdmission(denial))
            if denial.kind() == WorthQueryConvergenceEpochDenialKind::InvalidIncumbentTransition
    ));
}

pub fn request(identity: &str) -> WorthQueryManagedGraphCallRequest {
    WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, identity)
}

pub fn advance_direct(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    epoch: worth_query_execution::facade::convergence_epoch::WorthQueryIteratingDirectConvergenceEpoch,
    identity: &str,
) -> WorthQueryDirectConvergenceIterationOutcome {
    let started = {
        let installed_fixture_phase = epoch.fixture_phase_for_test(execution);
        epoch.begin_iteration(&installed_fixture_phase, request(identity))
    }
    .unwrap_or_else(|_| panic!("direct report history iteration must start"));
    {
        let fixture_phase_result = {
            let installed_fixture_phase = started.fixture_phase_for_test(execution);
            started.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
            _ => panic!("direct report history provider must complete"),
        }
    }
}

pub fn advance_workflow(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    epoch: worth_query_execution::facade::convergence_epoch::WorthQueryIteratingWorkflowConvergenceEpoch,
    identity: &str,
) -> WorthQueryWorkflowConvergenceIterationOutcome {
    let started = {
        let installed_fixture_phase = epoch.fixture_phase_for_test(execution);
        epoch.begin_stage_iteration(&installed_fixture_phase, WORKFLOW_STAGE, request(identity))
    }
    .unwrap_or_else(|_| panic!("workflow report history iteration must start"));
    {
        let fixture_phase_result = {
            let installed_fixture_phase = started.fixture_phase_for_test(execution);
            started.advance(&installed_fixture_phase)
        };
        match fixture_phase_result {
            WorthQueryWorkflowConvergenceStepOutcome::Completed(outcome) => outcome,
            _ => panic!("workflow report history provider must complete"),
        }
    }
}
