#[test]
fn dropping_the_arming_guard_clears_the_interleaving() {
    let guard = worth_query_execution::facade::domain_computation::WorthQueryExecutionObservation::install_on_this_thread();
    let world =
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    world
        .application
        .interleave_before_owned_reobservation_for_test(|| panic!("a dropped arm must never run"));
    drop(guard);
    let control = &(*(*(*(*world
        .application
        .runtime_for_test())
        .primary_provider_for_test())
        .application_attempt_operation_control_for_test())
        .caller_pass_for_test());
    control.run_interleaving_for_test();
}
