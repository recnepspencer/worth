//! current output source adjacency tracks actual anchor retirement and budget.
use super::*;

#[test]
fn current_output_source_adjacency_tracks_actual_anchor_retirement_and_budget() {
    let host_request = crate::test_request_policy::serial_request();
    let request_execution = &worth_execution::ExecutionRequest::serial(&host_request);

    use super::super::fixture::AccountOwner;
    use crate::domain_computation::primary_graph::application_attempt::{
        WorthQueryApplicationObservedFact as Fact, WorthQuerySourceCurrentnessFailure,
    };
    use worth_relational::facade::runtime::RelationalAdjacencyDirection;

    let world = installed_authorization_world(true);
    let (_, _, account, _) = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
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
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let fact = world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request(); graph.integration_handle().with_runtime(request_execution, |runtime| {
        let revision = runtime
            .read_truth()
            .project_snapshot(original.application_basis().snapshot_handle(), &host_custody.borrowed_request_for_test())
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
            fact.source_currentness_within(runtime,&host_custody.borrowed_request_for_test(),  snapshot, 1)
                .map(|(movement, work)| (movement.movement(), work)),
            Ok((
                crate::domain_computation::primary_graph::application_attempt::Movement::Unmoved,
                1
            ))
        );
        assert_eq!(
            fact.source_currentness_within(runtime,&host_custody.borrowed_request_for_test(),  snapshot, 0)
                .map(|(movement, work)| (movement.movement(), work)),
            Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)
        );
    }) }).expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            retire_open_account(&phase, &world)
        })
        .expect("the fixture read owns its installed host call");
    let retired = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    world.application.with_host_advancement(|phase| { let host_custody = phase.test_host_request(); graph.integration_handle().with_runtime(request_execution, |runtime| {
        let snapshot = retired.application_basis().snapshot_handle();
        assert_eq!(
            fact.source_currentness_within(runtime,&host_custody.borrowed_request_for_test(),  snapshot, 2)
                .map(|(movement, work)| (movement.movement(), work)),
            Ok((
                crate::domain_computation::primary_graph::application_attempt::Movement::Moved,
                2
            ))
        );
        assert_eq!(
            fact.source_currentness_within(runtime,&host_custody.borrowed_request_for_test(),  snapshot, 1)
                .map(|(movement, work)| (movement.movement(), work)),
            Err(WorthQuerySourceCurrentnessFailure::WorkBudgetExceeded)
        );
        // Retirement in a later basis does not rewrite the old source basis.
        assert_eq!(
            fact.source_currentness_within(
                runtime,&host_custody.borrowed_request_for_test(),
                original.application_basis().snapshot_handle(),
                1
            )
            .map(|(movement, work)| (movement.movement(), work)),
            Ok((
                crate::domain_computation::primary_graph::application_attempt::Movement::Unmoved,
                1
            ))
        );
    }) }).expect("the fixture read owns its installed host call");
}
