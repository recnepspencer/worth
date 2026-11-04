use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;

#[test]
fn primary_product_selection_composes_one_exact_application_snapshot() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    let world = installed_authorization_world(true);
    let lease = world
        .application
        .product_runtime()
        .admit_product_branch_for_test(
            world
                .application
                .product_runtime()
                .default_branch_for_test(),
        )
        .unwrap();
    let observer = world.application.application_query_basis_observer();
    let before = observer.observe();
    let expected = lease.relational_basis_descriptor().clone();
    let selected = world
        .application
        .on_product_for_test(execution, lease)
        .unwrap();
    assert_eq!(
        selected
            .application_basis_for_test()
            .identity()
            .descriptor(),
        &expected
    );
    assert_eq!(observer.observe().acquisitions(), before.acquisitions() + 1);
    assert!(
        (*(*world.application.primary_provider_for_test()).graph_for_test()).with_runtime(
            execution,
            |runtime| runtime
                .read_truth()
                .project_snapshot(selected.application_basis_for_test().snapshot_handle())
                .is_some()
        )
    );
    drop(selected);
    assert_eq!(observer.observe().active(), before.active());
}
