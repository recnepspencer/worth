//! current output source adjacency tracks actual anchor retirement.
use crate::domain_computation::primary_graph::tests::current_output_selection::*;

#[test]
fn current_output_source_adjacency_tracks_actual_anchor_retirement() {
    let host_request = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountOwner;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
    use worth_relational::facade::runtime::RelationalAdjacencyDirection;

    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let (_, _, account, _) = world
        .application
        .with_host_advancement(|phase| {
            admitted_touch_parts(&phase, &(phase).borrowed_request_for_test(), &world)
        })
        .expect("the fixture read owns its installed host call");
    let graph = (*world.application.runtime_for_test())
        .primary_graph()
        .unwrap();
    let kind = (*(*graph.layout_for_test())
        .relation(AccountOwner::reference().name())
        .unwrap()
        .kind_for_test());
    let original = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    let fact = world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); graph.integration_handle_for_test().with_runtime(request_execution, |runtime| {
        let revision = runtime
            .read_truth()
            .project_snapshot(original.application_basis_for_test().snapshot_handle())
            .unwrap()
            .bounded_adjacency_structural_revision(&host_custody.borrowed_request_for_test(),
                account.entity_id_for_test(),
                kind,
                RelationalAdjacencyDirection::Outgoing,
                1,
            )
            .unwrap();
        Fact::SourceAdjacencyRevision {
            relation_kind: kind,
            anchor: account.entity_id_for_test(),
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: revision.revision(),
            comparison_work_limit: 1,
            endpoints: worth_query_execution::facade::domain_computation::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&(Vec::new()), worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
        }
    }) }).expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); graph.integration_handle_for_test().with_runtime(request_execution, |runtime| {
        let snapshot = original.application_basis_for_test().snapshot_handle();
        assert_eq!(
            fact.source_currentness_in_for_test(runtime, &host_custody.borrowed_request_for_test(), snapshot).unwrap().map(|movement| movement.movement_for_test()),
            Ok(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::Movement::Unmoved)
        );

    }) }).expect("the fixture read owns its installed host call");
    retire_open_account(&world);
    let retired = world
        .application
        .with_host_advancement(|phase| {
            world.selected_product(&(phase).test_host_request_for_test())
        })
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request_for_test(); graph.integration_handle_for_test().with_runtime(request_execution, |runtime| {
        let snapshot = retired.application_basis_for_test().snapshot_handle();
        assert_eq!(
            fact.source_currentness_in_for_test(runtime, &host_custody.borrowed_request_for_test(), snapshot).unwrap().map(|movement| movement.movement_for_test()),
            Ok(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::Movement::Moved)
        );

        // Retirement in a later basis does not rewrite the old source basis.
        assert_eq!(
            fact.source_currentness_in_for_test(runtime, &host_custody.borrowed_request_for_test(), original.application_basis_for_test().snapshot_handle()).unwrap().map(|movement| movement.movement_for_test()),
            Ok(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::Movement::Unmoved)
        );
    }) }).expect("the fixture read owns its installed host call");
}
