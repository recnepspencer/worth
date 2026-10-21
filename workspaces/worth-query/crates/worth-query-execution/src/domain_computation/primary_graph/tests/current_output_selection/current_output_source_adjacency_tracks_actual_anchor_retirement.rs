//! current output source adjacency tracks actual anchor retirement.
use super::*;

#[test]
fn current_output_source_adjacency_tracks_actual_anchor_retirement() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use super::super::fixture::AccountOwner;
    use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact as Fact;
    use worth_relational::facade::runtime::RelationalAdjacencyDirection;

    let world = installed_authorization_world(true);
    let (_, _, account, _) = world
        .application
        .with_host_advancement(|phase| {
            admitted_touch_parts(&phase, &(phase).borrowed_request_for_test(), &world)
        })
        .expect("the fixture read owns its installed host call");
    let graph = world.application.runtime.primary_graph().unwrap();
    let kind = graph
        .layout
        .relation(AccountOwner::reference().name())
        .unwrap()
        .kind;
    let original = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    let fact = world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request(); graph.integration_handle().with_runtime(request_execution, |runtime| {
        let revision = runtime
            .read_truth()
            .project_snapshot(original.application_basis().snapshot_handle())
            .unwrap()
            .bounded_adjacency_structural_revision(&host_custody.borrowed_request_for_test(),
                account.entity_id(),
                kind,
                RelationalAdjacencyDirection::Outgoing,
                1,
            )
            .unwrap();
        Fact::SourceAdjacencyRevision {
            relation_kind: kind,
            anchor: account.entity_id(),
            direction: RelationalAdjacencyDirection::Outgoing,
            native_revision: revision.revision(),
            comparison_work_limit: 1,
            endpoints: crate::domain_computation::primary_graph::WorthQueryApplicationSourceAdjacencyEndpoints::from_observed(&(Vec::new()), worth_execution::ExecutionAllocationPolicy::SystemAllocation, None).unwrap(),
        }
    }) }).expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request(); graph.integration_handle().with_runtime(request_execution, |runtime| {
        let snapshot = original.application_basis().snapshot_handle();
        assert_eq!(
            fact.source_currentness_in(runtime, &host_custody.borrowed_request_for_test(), snapshot).unwrap().map(|movement| movement.movement()),
            Ok(crate::domain_computation::primary_graph::application_attempt::Movement::Unmoved)
        );

    }) }).expect("the fixture read owns its installed host call");
    retire_open_account(&world);
    let retired = world
        .application
        .with_host_advancement(|phase| world.selected_product(&(phase).test_host_request()))
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request(); graph.integration_handle().with_runtime(request_execution, |runtime| {
        let snapshot = retired.application_basis().snapshot_handle();
        assert_eq!(
            fact.source_currentness_in(runtime, &host_custody.borrowed_request_for_test(), snapshot).unwrap().map(|movement| movement.movement()),
            Ok(crate::domain_computation::primary_graph::application_attempt::Movement::Moved)
        );

        // Retirement in a later basis does not rewrite the old source basis.
        assert_eq!(
            fact.source_currentness_in(runtime, &host_custody.borrowed_request_for_test(), original.application_basis().snapshot_handle()).unwrap().map(|movement| movement.movement()),
            Ok(crate::domain_computation::primary_graph::application_attempt::Movement::Unmoved)
        );
    }) }).expect("the fixture read owns its installed host call");
}
