use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::precommit_snapshot::WorthQueryPrecommitSnapshot;

#[test]
fn unwind_releases_only_its_precommit_snapshot() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world =
            worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world(true, worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system());
    let selected = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let product = selected.product().publication_binding_for_test();
    let graph = (*(*(*world.application.runtime_for_test()).primary_provider_for_test())
        .graph_for_test())
    .clone();
    let count = || {
        graph.with_runtime(request_execution, |runtime| {
            runtime.retention().active_snapshot_count()
        })
    };
    let baseline = count();
    let peer = WorthQueryPrecommitSnapshot::acquire_for_test(
        request_execution,
        graph.clone(),
        product.observation_for_test().basis().relational_basis(),
    )
    .unwrap();
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _attempt = WorthQueryPrecommitSnapshot::acquire_for_test(
            request_execution,
            graph.clone(),
            product.observation_for_test().basis().relational_basis(),
        )
        .unwrap();
        assert_eq!(count(), baseline + 2);
        panic!("owner call interrupted before Query publication");
    }));
    assert!(unwind.is_err());
    assert_eq!(count(), baseline + 1);
    drop(peer);
    assert_eq!(count(), baseline);
}
