use super::fixture::{direct_epoch_fixture, FixtureDisposition};
use crate::domain_computation::{
    WorthQueryConvergenceTerminalKind, WorthQueryDirectConvergenceIterationOutcome,
    WorthQueryDirectConvergenceStepOutcome, WorthQueryGraphProviderCallKind,
    WorthQueryManagedGraphCallRequest, WorthQueryStartedDirectConvergenceIteration,
};

#[test]
fn real_installed_direct_authorities_progress_to_converged_and_cleanup() {
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
                "convergence-iteration",
            ),
        ) {
            Ok(started) => started,
            Err(_) => panic!("real installed graph authority must start the convergence iteration"),
        };
        let outcome = match started.advance(execution) {
            WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
            _ => panic!("single-step fixture provider must complete and rejoin its epoch"),
        };
        let terminal = match outcome {
            WorthQueryDirectConvergenceIterationOutcome::Converged(terminal) => terminal,
            _ => panic!("installed provider convergence decision must remain distinct"),
        };

        assert_eq!(
            terminal.kind(),
            WorthQueryConvergenceTerminalKind::Converged
        );
        assert_eq!(terminal.counters().iteration_count(), 1);
        assert_eq!(terminal.counters().provider_work_unit_count(), 1);
        assert_eq!(terminal.incumbents().len(), 1);
        let incumbent = &terminal.incumbents()[0];
        assert_ne!(incumbent.occurrence_identity(), "candidate-1");
        let report = terminal
            .latest_report()
            .expect("completed comparison must retain its report");
        assert_eq!(
            incumbent.report_evidence_identity(),
            report.evidence_identity()
        );
        assert_eq!(
            incumbent.state_identity(),
            report.decision().state_identity()
        );
        assert_eq!(report.decision().candidate_selection_key(), "candidate-1");
        let cleanup = match terminal.cleanup() {
            Ok(cleanup) => cleanup,
            Err(_) => panic!("converged direct epoch must retain cleanup authority"),
        };
        assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
        assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
    });
}

#[test]
fn same_semantic_candidate_in_real_direct_peers_has_distinct_occurrences() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let first = start_direct_peer(execution, "direct-peer", resource_request);
        let second = start_direct_peer(execution, "direct-peer", resource_request);
        let first = complete_direct_peer(execution, first);
        let second = complete_direct_peer(execution, second);

        assert_ne!(first, second);
    });
}

fn start_direct_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    scope: &str,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> WorthQueryStartedDirectConvergenceIteration {
    let epoch = direct_epoch_fixture(FixtureDisposition::Converged, resource_request);
    epoch
        .begin_iteration(
            execution,
            WorthQueryManagedGraphCallRequest::new(WorthQueryGraphProviderCallKind::Observe, scope),
        )
        .unwrap_or_else(|_| panic!("real direct peer must begin"))
}

fn complete_direct_peer(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    started: WorthQueryStartedDirectConvergenceIteration,
) -> String {
    let outcome = match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("real direct peer must complete"),
    };
    let terminal = match outcome {
        WorthQueryDirectConvergenceIterationOutcome::Converged(terminal) => terminal,
        _ => panic!("real direct peer must converge"),
    };
    assert_eq!(
        terminal
            .latest_report()
            .unwrap()
            .decision()
            .candidate_selection_key(),
        "candidate-1"
    );
    assert_eq!(
        terminal.incumbents()[0].report_evidence_identity(),
        terminal.latest_report().unwrap().evidence_identity()
    );
    terminal.incumbents()[0].occurrence_identity().to_owned()
}

#[test]
fn epoch_owned_iteration_bound_terminalizes_continue_as_exhausted() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let mut epoch = direct_epoch_fixture(FixtureDisposition::Continue, resource_request);
        for ordinal in 1..=3 {
            let started = match epoch.begin_iteration(
                execution,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Observe,
                    format!("convergence-iteration-{ordinal}"),
                ),
            ) {
                Ok(started) => started,
                Err(_) => panic!("bounded iteration must start"),
            };
            let outcome = match started.advance(execution) {
                WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
                _ => panic!("single-step fixture provider must complete and rejoin its epoch"),
            };
            match outcome {
                WorthQueryDirectConvergenceIterationOutcome::Continue(next) if ordinal < 3 => {
                    epoch = next;
                }
                WorthQueryDirectConvergenceIterationOutcome::Exhausted(terminal)
                    if ordinal == 3 =>
                {
                    assert_eq!(
                        terminal.kind(),
                        WorthQueryConvergenceTerminalKind::Exhausted
                    );
                    assert_eq!(terminal.counters().iteration_count(), 3);
                    assert_eq!(terminal.counters().provider_work_unit_count(), 3);
                    assert_eq!(terminal.incumbents().len(), 1);
                    assert_eq!(
                        terminal
                            .latest_report()
                            .expect("bounded iteration must retain its latest report")
                            .decision()
                            .candidate_selection_key(),
                        "candidate-3"
                    );
                    assert_ne!(
                        terminal.incumbents()[0].occurrence_identity(),
                        "candidate-3"
                    );
                    let cleanup = terminal
                        .cleanup()
                        .unwrap_or_else(|_| panic!("exhausted direct epoch must clean up"));
                    assert_eq!(cleanup.counters().cleanup_attempt_count(), 1);
                    assert_eq!(cleanup.counters().cleanup_completion_count(), 1);
                    return;
                }
                _ => panic!("iteration bound produced the wrong typed progression"),
            }
        }
        panic!("iteration bound did not terminalize the epoch")
    });
}
