use super::fixture::{direct_epoch_fixture, FixtureDisposition};
use super::terminal_fixture::{direct_terminal_outcome, indeterminate_terminal};
use crate::domain_computation::{
    WorthQueryConvergenceDomainInvocationFailureKind, WorthQueryConvergenceDomainPhase,
    WorthQueryConvergenceEpochDenialKind, WorthQueryConvergenceFeasibility,
    WorthQueryConvergenceIndeterminateCause, WorthQueryConvergenceProgress,
    WorthQueryConvergenceRepeatedState, WorthQueryConvergenceTerminalKind,
    WorthQueryConvergenceTerminalState, WorthQueryDirectConvergenceIterationOutcome,
    WorthQueryDirectConvergenceStepOutcome, WorthQueryDirectConvergenceTerminal,
    WorthQueryGraphProviderCallKind, WorthQueryManagedGraphCallRequest,
};
use worth_runtime_bridge::facade::BridgeManagedExecutionCancellationReason;

#[test]
fn installed_domain_semantics_preserve_distinct_terminal_kinds() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let cases = [
            (
                FixtureDisposition::Converged,
                WorthQueryConvergenceTerminalKind::Converged,
                1,
            ),
            (
                FixtureDisposition::StableWithoutProof,
                WorthQueryConvergenceTerminalKind::StableWithoutProof,
                1,
            ),
            (
                FixtureDisposition::FeasibleIncumbent,
                WorthQueryConvergenceTerminalKind::FeasibleIncumbent,
                1,
            ),
            (
                FixtureDisposition::Oscillating,
                WorthQueryConvergenceTerminalKind::Oscillating,
                0,
            ),
        ];
        for (fixture, expected, incumbent_count) in cases {
            match direct_terminal_outcome(execution, fixture, resource_request) {
                WorthQueryDirectConvergenceIterationOutcome::Converged(terminal) => {
                    assert_semantic_terminal(terminal, expected, incumbent_count)
                }
                WorthQueryDirectConvergenceIterationOutcome::StableWithoutProof(terminal) => {
                    assert_semantic_terminal(terminal, expected, incumbent_count)
                }
                WorthQueryDirectConvergenceIterationOutcome::FeasibleIncumbent(terminal) => {
                    assert_semantic_terminal(terminal, expected, incumbent_count)
                }
                WorthQueryDirectConvergenceIterationOutcome::Oscillating(terminal) => {
                    assert_semantic_terminal(terminal, expected, incumbent_count)
                }
                _ => panic!("semantic fixture reached the wrong terminal"),
            }
        }
    });
}

#[test]
fn incoherent_terminal_semantics_become_indeterminate_without_becoming_a_report() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let terminal = indeterminate_terminal(
            execution,
            FixtureDisposition::IncoherentStable,
            resource_request,
        );
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
        assert_eq!(terminal.incumbents().len(), 0);
        assert_eq!(terminal.counters().provider_work_unit_count(), 1);
        assert_eq!(terminal.counters().comparator_call_count(), 1);
        assert_eq!(terminal.counters().progress_check_count(), 1);
        assert_eq!(terminal.counters().repeated_state_probe_count(), 1);
        if terminal.cleanup().is_err() {
            panic!("rejected domain report must retain cleanup authority");
        }
    });
}

#[test]
fn stalled_progress_remains_explicit_domain_evidence() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let terminal =
            indeterminate_terminal(execution, FixtureDisposition::Stalled, resource_request);
        assert_eq!(
            terminal.kind(),
            WorthQueryConvergenceTerminalKind::Indeterminate
        );
        let report = terminal
            .latest_report()
            .expect("stalled progress must retain its admitted domain report");
        assert_eq!(
            report.decision().progress(),
            WorthQueryConvergenceProgress::Stalled
        );
        assert_eq!(
            report.decision().feasibility(),
            WorthQueryConvergenceFeasibility::Feasible
        );
        assert_eq!(terminal.incumbents().len(), 1);
        assert!(terminal.indeterminate_cause().is_none());
        if terminal.cleanup().is_err() {
            panic!("stalled convergence terminal must retain cleanup authority");
        }
    });
}

#[test]
fn indeterminate_comparison_retains_each_indeterminate_semantic_axis() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let terminal = indeterminate_terminal(
            execution,
            FixtureDisposition::IndeterminateComparison,
            resource_request,
        );
        assert_eq!(
            terminal.kind(),
            WorthQueryConvergenceTerminalKind::Indeterminate
        );
        let decision = terminal
            .latest_report()
            .expect("indeterminate comparison must retain its domain report")
            .decision();
        assert_eq!(
            decision.feasibility(),
            WorthQueryConvergenceFeasibility::Indeterminate
        );
        assert_eq!(
            decision.progress(),
            WorthQueryConvergenceProgress::Indeterminate
        );
        assert_eq!(
            decision.repeated_state(),
            WorthQueryConvergenceRepeatedState::Indeterminate
        );
        assert_eq!(terminal.incumbents().len(), 1);
        assert!(terminal.indeterminate_cause().is_none());
        if terminal.cleanup().is_err() {
            panic!("indeterminate comparison must retain cleanup authority");
        }
    });
}

#[test]
fn comparator_failure_retains_exact_attempted_work_without_admitting_a_report() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let terminal = indeterminate_terminal(
            execution,
            FixtureDisposition::ComparatorFailure,
            resource_request,
        );
        assert_eq!(
            terminal.kind(),
            WorthQueryConvergenceTerminalKind::Indeterminate
        );
        assert!(matches!(
            terminal.indeterminate_cause(),
            Some(WorthQueryConvergenceIndeterminateCause::DomainInvocation(failure))
                if failure.phase() == WorthQueryConvergenceDomainPhase::Comparator
                    && failure.kind() == WorthQueryConvergenceDomainInvocationFailureKind::Rejected
        ));
        assert!(terminal.latest_report().is_none());
        assert_eq!(terminal.counters().provider_work_unit_count(), 1);
        assert_eq!(terminal.counters().comparator_call_count(), 1);
        assert_eq!(terminal.counters().progress_check_count(), 0);
        assert_eq!(terminal.counters().repeated_state_probe_count(), 0);
        assert_eq!(terminal.incumbents().len(), 0);
        if terminal.cleanup().is_err() {
            panic!("comparator failure must retain cleanup authority");
        }
    });
}

#[test]
fn managed_signal_cancellation_remains_a_distinct_convergence_terminal() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let epoch = direct_epoch_fixture(FixtureDisposition::Converged, resource_request);
        let started = match epoch.begin_iteration(
            execution,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "cancelled-terminal",
            ),
        ) {
            Ok(started) => started,
            Err(_) => panic!("cancelled terminal iteration must start"),
        };
        started
            .request_cancellation(BridgeManagedExecutionCancellationReason::HostRequested)
            .expect("Signal cancellation request must admit");
        let outcome = match started.advance(execution) {
            WorthQueryDirectConvergenceStepOutcome::Terminal(outcome) => outcome,
            _ => panic!("Signal cancellation must terminalize and rejoin before provider work"),
        };
        let terminal = match outcome {
            WorthQueryDirectConvergenceIterationOutcome::Cancelled(terminal) => terminal,
            _ => panic!("cancelled managed run must remain a cancelled convergence terminal"),
        };
        assert_eq!(
            terminal.kind(),
            WorthQueryConvergenceTerminalKind::Cancelled
        );
        assert_eq!(terminal.counters().provider_work_unit_count(), 0);
        assert_eq!(terminal.incumbents().len(), 0);
        assert!(terminal.latest_report().is_none());
        let cleanup = match terminal.cleanup() {
            Ok(cleanup) => cleanup,
            Err(_) => panic!("cancelled convergence terminal must retain cleanup authority"),
        };
        assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
        assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
    });
}

fn assert_semantic_terminal<State>(
    terminal: WorthQueryDirectConvergenceTerminal<State>,
    expected: WorthQueryConvergenceTerminalKind,
    incumbent_count: usize,
) where
    State: WorthQueryConvergenceTerminalState,
{
    assert_eq!(terminal.kind(), expected);
    assert_eq!(terminal.counters().iteration_count(), 1);
    assert_eq!(terminal.counters().comparator_call_count(), 1);
    assert_eq!(terminal.counters().progress_check_count(), 1);
    assert_eq!(terminal.counters().repeated_state_probe_count(), 1);
    assert!(terminal.indeterminate_cause().is_none());
    assert_eq!(terminal.incumbents().len(), incumbent_count);
    let report = terminal
        .latest_report()
        .expect("semantic terminal must retain its admitted report");
    assert_eq!(report.domain_work().comparator_call_count(), 1);
    assert_eq!(report.domain_work().progress_check_count(), 1);
    assert_eq!(report.domain_work().repeated_state_probe_count(), 1);
    let cleanup = match terminal.cleanup() {
        Ok(cleanup) => cleanup,
        Err(_) => panic!("semantic terminal must retain cleanup authority"),
    };
    assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
    assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
}
