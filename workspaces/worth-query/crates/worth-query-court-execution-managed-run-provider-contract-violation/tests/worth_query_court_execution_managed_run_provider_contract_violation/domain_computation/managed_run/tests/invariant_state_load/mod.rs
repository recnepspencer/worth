use std::sync::Arc;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::execute_invariant;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::locator;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::requirement;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::state_with_outcome;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt_fixture::InvariantFixtureOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionDenialKind;
use worth_query_installation::facade::WorthQueryInvariantEnforcement;

#[test]
fn provider_must_load_exactly_the_admitted_locator_closure() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for (outcome, locators) in [
            (
                InvariantFixtureOutcome::OmitLoad,
                vec![locator("base"), locator("old")],
            ),
            (InvariantFixtureOutcome::OverreadLoad, vec![locator("base")]),
        ] {
            let state = state_with_outcome(outcome);
            let failure = execute_invariant(
                execution,
                Arc::clone(&state),
                vec![requirement(
                    "closed-loop",
                    WorthQueryInvariantEnforcement::Blocking,
                    4,
                )],
                "closed-loop",
                locators,
                resource_request,
            )
            .err()
            .expect("omitted or out-of-closure state must deny");
            assert_eq!(
                failure.kind(),
                WorthQueryInvariantExecutionDenialKind::StateLoadClosureMismatch
            );
            assert_eq!(state.lock().unwrap().invariant_load_calls, 1);
            assert_eq!(state.lock().unwrap().invariant_execution_calls, 0);
        }
    });
}
