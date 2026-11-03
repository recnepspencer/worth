use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::locator;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::requirement;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution::state;
use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::managed_run::tests::invariant_execution_fixture::execute_invariant_with_request;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantExecutionDenialKind as Kind;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantReceipt;
use std::time::Duration;
use std::time::Instant;
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_query_installation::facade::WorthQueryInvariantEnforcement;

#[test]
fn cancelled_invariant_request_stops_before_provider_state_load() {
    let requirement = requirement("closed-loop", WorthQueryInvariantEnforcement::Blocking, 1);
    let source = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        source.token(),
    );
    let passed = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|phase| {
        let bootstrap = phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        execute_invariant_with_request(
            &phase,
            state(),
            vec![requirement.clone()],
            "closed-loop",
            [locator("base")],
            Some(&request),
            resource_request,
        )
    })
    .result
    .unwrap();
    let WorthQueryInvariantReceipt::Passed(passed) = passed else {
        panic!("blocking actual provider passes")
    };
    assert_eq!(passed.counters().loaded_facts(), 1);
    assert_eq!(passed.counters().load_work_units(), 1);
    assert_eq!(passed.counters().execution_work_units(), 1);
    source.cancel();
    // A second host call: the cancelled request opens its own advancement.
    let stopped = state();
    let refusal = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|phase| {
        let bootstrap = phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        execute_invariant_with_request(
            &phase,
            stopped.clone(),
            vec![requirement],
            "closed-loop",
            [locator("base")],
            Some(&request),
            resource_request,
        )
    })
    .result
    .err()
    .unwrap();
    assert_eq!(
        refusal.kind(),
        Kind::RequestInterrupted(
            worth_relational::facade::mvcc::RelationalOperationInterruption::Cancelled
        )
    );
    assert_eq!(stopped.lock().unwrap().invariant_load_calls, 0);
    assert_eq!(stopped.lock().unwrap().invariant_execution_calls, 0);
}
