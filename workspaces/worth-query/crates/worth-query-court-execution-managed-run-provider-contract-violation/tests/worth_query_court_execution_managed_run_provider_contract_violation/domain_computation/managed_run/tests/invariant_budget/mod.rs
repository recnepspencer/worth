use std::num::NonZeroU32;

use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::execute_invariant;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::locator;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::state;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionFailurePosture;
use worth_query_installation::facade::WorthQueryInstalledInvariantExecutionRequirement;
use worth_query_installation::facade::WorthQueryInvariantEnforcement;

#[test]
fn state_load_and_validator_share_one_installed_work_budget() {
    worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let state = state();
        let requirement = WorthQueryInstalledInvariantExecutionRequirement::new(
            "closed-loop",
            "topology",
            NonZeroU32::new(1).unwrap(),
            WorthQueryInvariantEnforcement::Blocking,
            "managed-graph",
            ["region"],
            4,
            1,
        )
        .unwrap();
        let failure = execute_invariant(
            execution,
            std::sync::Arc::clone(&state),
            vec![requirement],
            "closed-loop",
            [locator("base")],
            resource_request,
        )
        .err()
        .expect("one load unit plus one execution unit must exceed a one-unit total budget");
        assert_eq!(
            failure.kind(),
            WorthQueryInvariantExecutionDenialKind::ExecutionBudgetExceeded
        );
        assert_eq!(
            failure.posture(),
            WorthQueryInvariantExecutionFailurePosture::Exhausted
        );
        let state = state.lock().unwrap();
        assert_eq!(state.invariant_load_calls, 1);
        assert_eq!(state.invariant_execution_calls, 1);
    });
}
