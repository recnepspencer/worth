use super::fixture::{
    direct_admission_fixture_with_contract_and_report_history_probe, FixtureConvergenceContract,
    FixtureDisposition, FixtureReportHistoryObservation,
};
use crate::domain_computation::{
    WorthQueryConverged, WorthQueryDirectConvergenceIterationOutcome,
    WorthQueryDirectConvergenceStepOutcome, WorthQueryDirectConvergenceTerminal,
    WorthQueryGraphProviderCallKind, WorthQueryIteratingDirectConvergenceEpoch,
    WorthQueryManagedGraphCallRequest, WorthQueryRetainedConvergenceCandidateEvidence,
};

#[test]
fn prepared_replace_commit_installs_one_exact_candidate() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (replace, replace_observations) = run_history(
            execution,
            FixtureDisposition::Converged,
            FixtureConvergenceContract::Bounded,
            1,
        );
        assert_eq!(replace.incumbents().len(), 1);
        assert_report_link(replace.incumbents(), &replace);
        assert_eq!(replace.counters().incumbent_retention_count(), 0);
        assert_eq!(replace.counters().incumbent_replacement_count(), 1);
        assert_eq!(replace_observations.len(), 1);
        assert!(replace.cleanup().is_ok());
    });
}

#[test]
fn prepared_retain_commit_keeps_the_exact_prior_incumbent() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (retain, retain_observations) = run_history(
            execution,
            FixtureDisposition::HistoryRetain,
            FixtureConvergenceContract::Bounded,
            2,
        );
        let retained_before = &retain_observations[1].incumbents()[0];
        assert_eq!(retain.incumbents().len(), 1);
        assert_eq!(
            retain.incumbents()[0].occurrence_identity(),
            retained_before.occurrence_identity()
        );
        assert_eq!(
            retain.incumbents()[0].state_identity(),
            retained_before.state_identity()
        );
        assert_eq!(
            retain.incumbents()[0].report_evidence_identity(),
            retained_before.report_evidence_identity()
        );
        assert_ne!(
            retain.incumbents()[0].report_evidence_identity(),
            retain.latest_report().unwrap().evidence_identity()
        );
        assert_eq!(retain.counters().incumbent_retention_count(), 1);
        assert_eq!(retain.counters().incumbent_replacement_count(), 1);
        assert!(retain.cleanup().is_ok());
    });
}

#[test]
fn prepared_add_commit_preserves_prior_candidates() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (add, _) = run_history(
            execution,
            FixtureDisposition::ParetoCollision,
            FixtureConvergenceContract::Pareto,
            2,
        );
        assert_eq!(add.incumbents().len(), 2);
        assert_report_link(add.incumbents(), &add);
        assert_eq!(add.counters().incumbent_replacement_count(), 2);
        assert!(add.cleanup().is_ok());
    });
}

#[test]
fn prepared_remove_and_add_commit_removes_only_the_named_candidate() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (remove_and_add, remove_observations) = run_history(
            execution,
            FixtureDisposition::ParetoPartialReplacement,
            FixtureConvergenceContract::Pareto,
            3,
        );
        let before_removal = remove_observations[2].incumbents();
        assert_eq!(before_removal.len(), 2);
        assert_eq!(remove_and_add.incumbents().len(), 2);
        assert!(!contains_occurrence(
            remove_and_add.incumbents(),
            before_removal[0].occurrence_identity()
        ));
        assert!(contains_occurrence(
            remove_and_add.incumbents(),
            before_removal[1].occurrence_identity()
        ));
        assert_report_link(remove_and_add.incumbents(), &remove_and_add);
        assert_eq!(remove_and_add.counters().incumbent_replacement_count(), 3);
        assert!(remove_and_add.cleanup().is_ok());
    });
}

#[test]
fn prepared_clear_commit_empties_incumbents_but_installs_the_report() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (clear, clear_observations) = run_history(
            execution,
            FixtureDisposition::HistoryClear,
            FixtureConvergenceContract::Bounded,
            2,
        );
        assert_eq!(clear_observations[1].incumbents().len(), 1);
        assert!(clear.incumbents().is_empty());
        assert_eq!(clear.latest_report().unwrap().iteration_ordinal(), 2);
        assert_eq!(clear.counters().incumbent_retention_count(), 0);
        assert_eq!(clear.counters().incumbent_replacement_count(), 2);
        assert!(clear.cleanup().is_ok());
    });
}

fn run_history(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    disposition: FixtureDisposition,
    contract: FixtureConvergenceContract,
    iteration_count: usize,
) -> (
    WorthQueryDirectConvergenceTerminal<WorthQueryConverged>,
    Vec<FixtureReportHistoryObservation>,
) {
    let (fixture, probe) =
        direct_admission_fixture_with_contract_and_report_history_probe(disposition, contract);
    let mut epoch = fixture.admit();
    for ordinal in 1..=iteration_count {
        let outcome = advance(execution, epoch, &format!("report-history-{ordinal}"));
        if ordinal == iteration_count {
            let terminal = match outcome {
                WorthQueryDirectConvergenceIterationOutcome::Converged(terminal) => terminal,
                _ => panic!("final report history transition must converge"),
            };
            return (terminal, probe.observations());
        }
        epoch = match outcome {
            WorthQueryDirectConvergenceIterationOutcome::Continue(epoch) => epoch,
            _ => panic!("intermediate report history transition must continue"),
        };
    }
    unreachable!("report history scenario must execute at least one iteration")
}

fn advance(
    execution: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,

    epoch: WorthQueryIteratingDirectConvergenceEpoch,
    call_identity: &str,
) -> WorthQueryDirectConvergenceIterationOutcome {
    let started = epoch
        .begin_iteration(
            execution,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                call_identity,
            ),
        )
        .unwrap_or_else(|_| panic!("report history iteration must start"));
    match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("report history provider must complete in one step"),
    }
}

fn assert_report_link(
    incumbents: &[WorthQueryRetainedConvergenceCandidateEvidence],
    terminal: &WorthQueryDirectConvergenceTerminal<WorthQueryConverged>,
) {
    let report_identity = terminal.latest_report().unwrap().evidence_identity();
    assert_eq!(
        incumbents
            .iter()
            .filter(|candidate| candidate.report_evidence_identity() == report_identity)
            .count(),
        1
    );
}

fn contains_occurrence(
    incumbents: &[WorthQueryRetainedConvergenceCandidateEvidence],
    occurrence_identity: &str,
) -> bool {
    incumbents
        .iter()
        .any(|candidate| candidate.occurrence_identity() == occurrence_identity)
}
