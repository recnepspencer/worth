//! A real World member retirement preserves its component name and invalidates membership.
use crate::domain_computation::primary_graph::invariant_projection::component::tests::*;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation_on_application;
use worth_relational::facade::transactions::DeleteEntityIntent;
use worth_relational::facade::transactions::EntityMutationIntent;
use worth_relational::facade::transactions::MutationIntent;
use worth_relational::facade::transactions::WorkerIntentBatch;

#[test]
fn member_retirement_preserves_component_name_and_invalidates_old_membership() {
    let _isolation =
        worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = install();
    let prior = world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let removed = reader.resolve_entity(NodeKey::reference(), 2_u64).unwrap();
                        let root = reader.resolve_entity(NodeKey::reference(), 1_u64).unwrap();
                        let component = reader.component_of(Shells::reference(), &root).unwrap();
                        (removed.entity_id(), component)
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            publish_relational_mutation_on_application(
                &host.borrowed_request_for_test(),
                &world.application,
                WorkerIntentBatch::new("retained-component-edge").push(MutationIntent::Entity(
                    EntityMutationIntent::Delete(DeleteEntityIntent {
                        entity_id: prior.output().0,
                    }),
                )),
            );
        })
        .unwrap();
    world.application.with_host_advancement(|phase| {
        world.invariant.project(&phase, |reader| {
            let root = reader.resolve_entity(NodeKey::reference(), 1_u64).unwrap();
            // Ordinary coupling excludes an audit-retained relation to a retired member.
            let kind = (*(*reader.layout_for_test()).relation("B").unwrap().kind_for_test());
            let physical = reader.read_adjacency_for_test("B", root.entity_id(), kind,
                worth_query_declaration::facade::application_schema::RelationDirection::Forward).unwrap();
            assert!(physical.into_records().is_empty(), "the authored B1->2 edge is no longer live coupling");
            let current = reader.component_of(Shells::reference(), &root).unwrap();
            let keys = current.members().map(|member| reader.field(&member, NodeKey::reference()).unwrap())
                .collect::<BTreeSet<_>>();
            assert_eq!(keys, BTreeSet::from([0, 1]));
            assert_eq!(current.identity(), prior.output().1.identity(), "retirement without a split preserves the persistent name");
            let pin = (*reader.native_for_test()).committed_runtime_for_test().expect("committed fixture read").read_truth().pin_read_capability(reader.snapshot_for_test(), &[], reader.execution_for_test()).unwrap();
            assert_ne!(pin.snapshot().version_id(), prior.output().1.aggregate().source_for_test().committed_snapshot_for_test().expect("committed fixture read").version_id());
            let equal = (*reader.execution_for_test()).run_native_with_checkpoint(|context|
                prior.output().1.aggregate().remains_equal_at_for_test(&pin, reader.execution_for_test(), context, None).unwrap()).unwrap();
            assert!(!equal.0, "actual changed-product membership cannot reuse the old component answer");
        }, Allocation::SystemAllocation).unwrap();
    }).unwrap();
    world.application.close_live_delivery();
}
