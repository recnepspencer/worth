use super::fixture::{
    direct_admission_fixture_with_contract, FixtureConvergenceContract, FixtureDisposition,
};
use crate::domain_computation::{
    WorthQueryConvergenceEpochDenialKind, WorthQueryConvergenceIndeterminateCause,
    WorthQueryConvergenceTerminalKind, WorthQueryDirectConvergenceIterationOutcome,
    WorthQueryDirectConvergenceStepOutcome, WorthQueryDirectConvergenceTerminal,
    WorthQueryGraphProviderCallKind, WorthQueryIndeterminate, WorthQueryManagedGraphCallRequest,
    WorthQueryOscillating,
};

#[test]
fn installed_repeated_state_policies_reject_contradictory_domain_reports() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let impossible = indeterminate_terminal(
            execution,
            FixtureDisposition::OscillatingSelected,
            FixtureConvergenceContract::OscillationImpossible,
        );
        assert_indeterminate_policy_denial(impossible);

        let continued_repetition = indeterminate_terminal(
            execution,
            FixtureDisposition::RepeatedContinue,
            FixtureConvergenceContract::Bounded,
        );
        assert_indeterminate_policy_denial(continued_repetition);
    });
}

#[test]
fn installed_oscillation_postures_govern_incumbent_selection() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let denied = oscillating_terminal(
            execution,
            FixtureDisposition::Oscillating,
            FixtureConvergenceContract::Bounded,
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

fn assert_indeterminate_policy_denial(
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

fn indeterminate_terminal(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    disposition: FixtureDisposition,
    contract: FixtureConvergenceContract,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryIndeterminate> {
    match terminal_outcome(execution, disposition, contract) {
        WorthQueryDirectConvergenceIterationOutcome::Indeterminate(terminal) => terminal,
        _ => panic!("oscillation policy fixture must be indeterminate"),
    }
}

fn oscillating_terminal(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    disposition: FixtureDisposition,
    contract: FixtureConvergenceContract,
) -> WorthQueryDirectConvergenceTerminal<WorthQueryOscillating> {
    match terminal_outcome(execution, disposition, contract) {
        WorthQueryDirectConvergenceIterationOutcome::Oscillating(terminal) => terminal,
        _ => panic!("oscillation policy fixture must be oscillating"),
    }
}

fn terminal_outcome(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    disposition: FixtureDisposition,
    contract: FixtureConvergenceContract,
) -> WorthQueryDirectConvergenceIterationOutcome {
    let epoch = direct_admission_fixture_with_contract(disposition, contract).admit();
    let started = match epoch.begin_iteration(
        execution,
        WorthQueryManagedGraphCallRequest::new(
            WorthQueryGraphProviderCallKind::Observe,
            "oscillation-policy",
        ),
    ) {
        Ok(started) => started,
        Err(_) => panic!("oscillation policy fixture iteration must start"),
    };
    let outcome = match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("oscillation policy fixture provider must complete and rejoin its epoch"),
    };
    outcome
}
