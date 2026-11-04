use std::sync::Arc;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::execute_invariant;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::locator;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::requirement;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::state_with_outcome;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::provisional_attempt_fixture::InvariantFixtureOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionFailurePosture;
use worth_query_installation::facade::WorthQueryInvariantEnforcement;

#[test]
fn provider_cannot_replay_a_retained_verdict_admission_into_another_attempt() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let hostile = state_with_outcome(InvariantFixtureOutcome::RetainThenSubstitute);
        let first = execute_invariant(
            execution,
            Arc::clone(&hostile),
            vec![requirement(
                "closed-loop",
                WorthQueryInvariantEnforcement::Blocking,
                4,
            )],
            "closed-loop",
            [locator("base")],
            resource_request,
        )
        .err()
        .expect("hostile provider must retain the first admission");
        assert_eq!(
            first.kind(),
            WorthQueryInvariantExecutionDenialKind::ProviderRejected
        );

        let substituted = execute_invariant(
            execution,
            hostile,
            vec![requirement(
                "closed-loop",
                WorthQueryInvariantEnforcement::Blocking,
                4,
            )],
            "closed-loop",
            [locator("base")],
            resource_request,
        )
        .err()
        .expect("retained admission must not authorize another attempt");
        assert_eq!(
            substituted.kind(),
            WorthQueryInvariantExecutionDenialKind::EvidenceSubstitution
        );
    });
}

#[test]
fn provider_panics_during_load_and_execution_are_typed() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        for outcome in [
            InvariantFixtureOutcome::PanicDuringLoad,
            InvariantFixtureOutcome::PanicDuringExecution,
        ] {
            let failure = execute_invariant(
                execution,
                state_with_outcome(outcome),
                vec![requirement(
                    "closed-loop",
                    WorthQueryInvariantEnforcement::Blocking,
                    4,
                )],
                "closed-loop",
                [locator("base")],
                resource_request,
            )
            .err()
            .expect("provider panic must become a typed invariant failure");
            assert_eq!(
                failure.kind(),
                WorthQueryInvariantExecutionDenialKind::ProviderPanicked
            );
            assert_eq!(
                failure.posture(),
                WorthQueryInvariantExecutionFailurePosture::Denied
            );
        }
    });
}
