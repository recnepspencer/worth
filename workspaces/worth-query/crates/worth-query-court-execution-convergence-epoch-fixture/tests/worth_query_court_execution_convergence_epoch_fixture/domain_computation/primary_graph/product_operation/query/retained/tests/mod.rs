use std::num::NonZeroUsize;

use worth_query_execution::facade::domain_computation::WorthQueryApplicationQueryAdmissionDenialKind;
    use worth_query_execution::facade::domain_computation::WorthQueryProductQueryControls;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;

#[test]
fn retained_and_batch_selection_keep_original_interruption_causes() {
    use worth_query_execution::facade::domain_computation::WorthQueryProductBranchAdmissionDenial as Branch;
    use worth_query_execution::facade::domain_computation::WorthQueryProviderSessionControlStopKind as Stop;
    for (stop, expected) in [
        (
            Stop::Cancelled,
            WorthQueryApplicationQueryAdmissionDenialKind::Cancelled,
        ),
        (
            Stop::TimedOut,
            WorthQueryApplicationQueryAdmissionDenialKind::DeadlineExceeded,
        ),
    ] {
        let denial = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::product_operation::query::selection_denial(Branch::Stopped(Err(stop)), "batch-stop");
        assert_eq!(denial.kind(), expected);
    }
}

#[test]
fn retained_query_rejects_security_from_a_sibling_branch() {
    let world = installed_authorization_world(true);
    let application = &world.application;
    let root = application.current_world();
    let sibling = world
        .application
        .with_host_advancement(|phase| {
            application
                .branches()
                .fork(root)
                .components(|components| components.reuse_exact_relational_basis().fork_signal())
                .create(&phase)
                .expect("a distinct sibling branch should exist")
        })
        .expect("the fixture read owns its installed host call");
    let security = world
        .application
        .with_host_advancement(|phase| application.on_branch(sibling).select(&phase).unwrap())
        .expect("the fixture read owns its installed host call");
    let retained = world
        .application
        .with_host_advancement(|phase| application.on_branch(root).select(&phase).unwrap())
        .expect("the fixture read owns its installed host call");
    let scope = live_scope();
    let controls = WorthQueryProductQueryControls::new(NonZeroUsize::new(1).unwrap(), &scope);
    let denied = application
        .with_host_advancement(|phase| {
            let execution = phase
                .execution_request_for_for_test(&(*application.product_runtime_for_test()))
                .unwrap();
            security.retained_query_controls_for_test(&execution, retained, "sibling", controls)
        })
        .expect("original fixture host call");
    assert!(matches!(
        denied,
        Err(error)
            if error.kind() == WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis
    ));
}
