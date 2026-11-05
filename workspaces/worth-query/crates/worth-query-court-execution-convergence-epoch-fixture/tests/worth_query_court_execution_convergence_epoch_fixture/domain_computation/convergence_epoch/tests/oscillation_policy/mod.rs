use crate::domain_computation::convergence_epoch::tests::fixture::candidate_contract::FixtureConvergenceContract;
use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_admission_fixture_with_contract;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
use worth_query_execution::court_doors_for_test::domain_computation::WorthQueryIndeterminate;
use worth_query_execution::court_doors_for_test::domain_computation::WorthQueryOscillating;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceEpochDenialKind;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceIndeterminateCause;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceTerminalKind;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceTerminal;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;

#[test]
fn installed_repeated_state_policies_reject_contradictory_domain_reports() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let impossible = indeterminate_terminal(
            execution,
            FixtureDisposition::OscillatingSelected,
            FixtureConvergenceContract::OscillationImpossible,
            resource_request,
        );
        assert_indeterminate_policy_denial(impossible);

        let continued_repetition = indeterminate_terminal(
            execution,
            FixtureDisposition::RepeatedContinue,
            FixtureConvergenceContract::Bounded,
            resource_request,
        );
        assert_indeterminate_policy_denial(continued_repetition);
    });
}

#[test]
fn installed_oscillation_postures_govern_incumbent_selection() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let denied = oscillating_terminal(
            execution,
            FixtureDisposition::Oscillating,
            FixtureConvergenceContract::Bounded,
            resource_request,
        );
        assert_eq!(
            denied.kind(),
            WorthQueryConvergenceTerminalKind::Oscillating
        );
        assert!(denied.incumbents().is_empty());
        assert!(denied.latest_report().is_some());
        assert!(denied.indeterminate_cause().is_none());
        if denied.cleanup().is_err() {
            panic!("detect-and-deny oscillation must retain cleanup authority");
        }

        let selected = oscillating_terminal(
            execution,
            FixtureDisposition::OscillatingSelected,
            FixtureConvergenceContract::OscillationSelectIncumbent,
            resource_request,
        );
        assert_eq!(
            selected.kind(),
            WorthQueryConvergenceTerminalKind::Oscillating
        );
        assert_eq!(selected.incumbents().len(), 1);
        assert!(selected.latest_report().is_some());
        assert!(selected.indeterminate_cause().is_none());
        if selected.cleanup().is_err() {
            panic!("detect-and-select oscillation must retain cleanup authority");
        }

        let classified = oscillating_terminal(
            execution,
            FixtureDisposition::DomainClassifiedOscillation,
            FixtureConvergenceContract::OscillationDomainClassified,
            resource_request,
        );
        assert_eq!(
            classified.kind(),
            WorthQueryConvergenceTerminalKind::Oscillating
        );
        assert_eq!(classified.incumbents().len(), 1);
        assert!(classified.latest_report().is_some());
        assert!(classified.indeterminate_cause().is_none());
        if classified.cleanup().is_err() {
            panic!("domain-classified oscillation must retain cleanup authority");
        }
    });
}

pub fn assert_indeterminate_policy_denial(
    terminal: WorthQueryDirectConvergenceTerminal<WorthQueryIndeterminate>,
) {
    assert_eq!(
        terminal.kind(),
        WorthQueryConvergenceTerminalKind::Indeterminate
    );
    assert!(matches!(
        terminal.indeterminate_cause(),
        Some(WorthQueryConvergenceIndeterminateCause::ReportAdmission(denial))
            if denial.kind() == WorthQueryConvergenceEpochDenialKind::InvalidDomainReport
    ));
    assert!(terminal.latest_report().is_none());
    assert!(terminal.incumbents().is_empty());
    if terminal.cleanup().is_err() {
        panic!("rejected oscillation report must retain cleanup authority");
    }
}

pub fn indeterminate_terminal(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    disposition: FixtureDisposition,
    contract: FixtureConvergenceContract,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryIndeterminate> {
    match terminal_outcome(execution, disposition, contract, resource_request) {
        WorthQueryDirectConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
        _ => panic!("oscillation policy fixture must be indeterminate"),
    }
}

pub fn oscillating_terminal(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    disposition: FixtureDisposition,
    contract: FixtureConvergenceContract,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryOscillating> {
    match terminal_outcome(execution, disposition, contract, resource_request) {
        WorthQueryDirectConvergenceIterationOutcome::Oscillating(terminal) => terminal,
        _ => panic!("oscillation policy fixture must be oscillating"),
    }
}

pub fn terminal_outcome(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    disposition: FixtureDisposition,
    contract: FixtureConvergenceContract,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryDirectConvergenceIterationOutcome {
    let epoch =
        direct_admission_fixture_with_contract(disposition, contract, resource_request).admit();
    let started = {
        let fixture_phase_result = {
            let fixture_target = epoch;
            let installed_fixture_phase = fixture_target.fixture_phase_for_test(execution);
            fixture_target.begin_iteration(
                &installed_fixture_phase,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    "oscillation-policy",
                ),
            )
        };
        match fixture_phase_result {
            Ok(started) => started,
            Err(_) => panic!("oscillation policy fixture iteration must start"),
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
            _ => panic!("oscillation policy fixture provider must complete and rejoin its epoch"),
        }
    }
}
