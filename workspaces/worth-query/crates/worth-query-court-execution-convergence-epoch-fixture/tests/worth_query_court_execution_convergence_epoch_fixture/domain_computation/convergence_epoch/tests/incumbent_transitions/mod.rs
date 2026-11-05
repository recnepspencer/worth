use crate::domain_computation::convergence_epoch::tests::fixture::candidate_contract::FixtureConvergenceContract;
use crate::domain_computation::convergence_epoch::tests::fixture::direct_installed_world::direct_admission_fixture_with_contract;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
use worth_query_execution::facade::convergence_epoch::WorthQueryConvergenceTerminalKind;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceIterationOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryDirectConvergenceStepOutcome;
use worth_query_execution::facade::convergence_epoch::WorthQueryIteratingDirectConvergenceEpoch;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCallKind;
use worth_query_execution::facade::domain_computation::WorthQueryManagedGraphCallRequest;

#[test]
fn pareto_remove_and_add_commits_only_after_the_installed_transition_validates() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let epoch = direct_admission_fixture_with_contract(
            FixtureDisposition::ParetoReplacement,
            FixtureConvergenceContract::Pareto,
            resource_request,
        )
        .admit();
        let epoch = match advance(execution, epoch, "pareto-add") {
            WorthQueryDirectConvergenceIterationOutcome::Continue(epoch) => epoch,
            _ => panic!("first Pareto candidate must continue with retained incumbent authority"),
        };
        let terminal = match advance(execution, epoch, "pareto-replace") {
            WorthQueryDirectConvergenceIterationOutcome::Converged(terminal) => terminal,
            _ => panic!("valid Pareto replacement must preserve the converged terminal"),
        };

        assert_eq!(
            terminal.kind(),
            WorthQueryConvergenceTerminalKind::Converged
        );
        assert_eq!(terminal.counters().iteration_count(), 2);
        assert_eq!(terminal.counters().incumbent_retention_count(), 0);
        assert_eq!(terminal.counters().incumbent_replacement_count(), 2);
        assert_eq!(terminal.incumbents().len(), 1);
        assert_ne!(
            terminal.incumbents()[0].occurrence_identity(),
            "candidate-2"
        );
        let report = terminal
            .latest_report()
            .expect("valid replacement must admit its report");
        assert_eq!(report.decision().candidate_selection_key(), "candidate-2");
        assert_eq!(
            terminal.incumbents()[0].report_evidence_identity(),
            report.evidence_identity()
        );
        if terminal.cleanup().is_err() {
            panic!("valid Pareto replacement must retain cleanup authority");
        }
    });
}

#[test]
fn repeated_semantic_candidate_key_mints_a_distinct_pareto_occurrence() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let epoch = direct_admission_fixture_with_contract(
            FixtureDisposition::ParetoCollision,
            FixtureConvergenceContract::Pareto,
            resource_request,
        )
        .admit();
        let epoch = match advance(execution, epoch, "pareto-first") {
            WorthQueryDirectConvergenceIterationOutcome::Continue(epoch) => epoch,
            _ => panic!("first Pareto candidate must be retained"),
        };
        let terminal = match advance(execution, epoch, "pareto-same-semantic-key") {
            WorthQueryDirectConvergenceIterationOutcome::Converged(terminal) => terminal,
            _ => panic!("a fresh execution of the same semantic candidate must remain distinct"),
        };

        assert_eq!(
            terminal.kind(),
            WorthQueryConvergenceTerminalKind::Converged
        );
        assert_eq!(terminal.incumbents().len(), 2);
        assert_eq!(terminal.counters().incumbent_retention_count(), 0);
        assert_eq!(terminal.counters().incumbent_replacement_count(), 2);
        assert_ne!(
            terminal.incumbents()[0].occurrence_identity(),
            terminal.incumbents()[1].occurrence_identity()
        );
        let report = terminal
            .latest_report()
            .expect("the second exact execution must retain its report");
        assert_eq!(
            report.decision().candidate_selection_key(),
            "candidate-pareto"
        );
        assert_eq!(
            terminal.incumbents()[1].report_evidence_identity(),
            report.evidence_identity()
        );
        if terminal.cleanup().is_err() {
            panic!("distinct Pareto occurrences must retain cleanup authority");
        }
    });
}

pub fn advance(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    epoch: WorthQueryIteratingDirectConvergenceEpoch,
    call_identity: &str,
) -> WorthQueryDirectConvergenceIterationOutcome {
    let installed_fixture_phase = epoch.fixture_phase_for_test(execution);
    let execution = &installed_fixture_phase;

    let started = match epoch.begin_iteration(
        execution,
        WorthQueryManagedGraphCallRequest::new(
            WorthQueryGraphProviderCallKind::Observe,
            call_identity,
        ),
    ) {
        Ok(started) => started,
        Err(_) => panic!("installed Pareto epoch must begin its next iteration"),
    };
    match started.advance(execution) {
        WorthQueryDirectConvergenceStepOutcome::Completed(outcome) => outcome,
        _ => panic!("Pareto fixture graph provider must complete and rejoin in one step"),
    }
}
