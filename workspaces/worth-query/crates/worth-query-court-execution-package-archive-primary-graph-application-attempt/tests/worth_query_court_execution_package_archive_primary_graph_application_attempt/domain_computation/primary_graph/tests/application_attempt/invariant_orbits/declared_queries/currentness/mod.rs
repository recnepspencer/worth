//! Query row/set retention preserves original aggregate and walk bodies independently.
use crate::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::*;
use worth_query_execution::domain_computation::primary_graph::application_attempt::snapshot_lease::WorthQueryApplicationSnapshotLease;
use std::sync::Arc;
use worth_execution::ExecutionAllocationPolicy as Allocation;
use worth_execution::ExecutionRequest;

#[test]
fn retained_query_aggregate_identity_is_not_hidden_by_its_walk_companion() {
    let _isolation =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = world();
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let request = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope();
    let results = [
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Orbit::reference(), 0),
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::declared_queries::read::run(&world, Orbit::reference(), 3),
    ];
    let bodies = results.each_ref().map(|result| {
        let layout = world.application.runtime.primary_graph().unwrap().layout();
        let facts = result.result_set_observation().source().retained_checkpoint_facts(layout, Some(&request), Allocation::SystemAllocation).unwrap();
        let aggregate = facts.iter().find_map(|fact| match fact { WorthQueryApplicationObservedFact::Aggregate { observation } => Some(Arc::clone(observation)), _ => None }).unwrap();
        let walk = facts.iter().find_map(|fact| match fact { WorthQueryApplicationObservedFact::StructuralWalk { observation } => Some(Arc::clone(observation)), _ => None }).unwrap();
        assert_eq!(facts.iter().filter(|fact| matches!(fact, WorthQueryApplicationObservedFact::Aggregate { .. } | WorthQueryApplicationObservedFact::StructuralWalk { .. })).count(), 2);
        for source in result.observed_sources() {
            let row = source.retained_checkpoint_facts(layout, Some(&request), Allocation::SystemAllocation).unwrap();
            assert!(row.iter().any(|fact| matches!(fact, WorthQueryApplicationObservedFact::Aggregate { observation } if Arc::ptr_eq(observation, &aggregate))));
            assert!(row.iter().any(|fact| matches!(fact, WorthQueryApplicationObservedFact::StructuralWalk { observation } if Arc::ptr_eq(observation, &walk))));
        }
        assert_eq!(walk.native().visits()[0].entity.entity_id, aggregate.native().anchor(), "the actual Native anchor determines the first row");
        (aggregate, walk)
    });
    let winner = usize::from(bodies[0].0.native().anchor() > bodies[1].0.native().anchor());
    for key in [245, 246] {
        let change = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::program::swap_successors(&world, &request, 0, 3);
        assert!(matches!(
            worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::commit(&world, &request, change, key),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ));
    }
    let lease = snapshot(&world, &execution);
    for (index, (aggregate, walk)) in bodies.iter().enumerate() {
        lease.handle().with_runtime(&execution, |runtime| {
            let mut extent = None;
            let aggregate_fact = WorthQueryApplicationObservedFact::Aggregate { observation: Arc::clone(aggregate) };
            let movement = aggregate_fact.source_currentness_with(runtime, &execution, lease.snapshot(), &mut extent, None).unwrap().unwrap().movement();
            assert_eq!(movement == worth_query_execution::domain_computation::primary_graph::application_attempt::fact::movement::Movement::Unmoved, index == winner);
            let again = aggregate_fact.source_currentness_with(runtime, &execution, lease.snapshot(), &mut extent, None).unwrap().unwrap().movement();
            assert_eq!(movement, again);
            let walk_fact = WorthQueryApplicationObservedFact::StructuralWalk { observation: Arc::clone(walk) };
            assert_eq!(walk_fact.source_currentness_with(runtime, &execution, lease.snapshot(), &mut extent, None).unwrap().unwrap().movement(), worth_query_execution::domain_computation::primary_graph::application_attempt::fact::movement::Movement::Moved);
        });
    }
}
pub fn snapshot(
    world: &World,
    execution: &ExecutionRequest<'_, '_>,
) -> WorthQueryApplicationSnapshotLease {
    let selected = world
        .application
        .select_product_branch(
            execution,
            world.application.product_runtime().default_branch(),
        )
        .unwrap();
    let completed = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(&phase, |_| (), Allocation::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    let ((), snapshot, _) = completed.into_parts();
    snapshot
        .into_lease_and_realized_scope(selected.product().retained_clone())
        .0
}
