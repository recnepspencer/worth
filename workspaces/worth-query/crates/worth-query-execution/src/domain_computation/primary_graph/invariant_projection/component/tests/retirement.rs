//! A real World member retirement preserves its component name and invalidates membership.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::publish_relational_mutation_on_application;
use worth_relational::facade::transactions::{
    DeleteEntityIntent, EntityMutationIntent, MutationIntent, WorkerIntentBatch,
};

#[test]
fn member_retirement_preserves_component_name_and_invalidates_old_membership() {
    let _isolation =
        crate::domain_computation::primary_graph::tests::fixture::isolated_request_owner();
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
            let host = phase.test_host_request();
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
            let kind = reader.layout.relation("B").unwrap().kind;
            let physical = reader.read_adjacency("B", root.entity_id(), kind,
                worth_query_declaration::facade::application_schema::RelationDirection::Forward).unwrap();
            assert!(physical.into_records().is_empty(), "the authored B1->2 edge is no longer live coupling");
            let current = reader.component_of(Shells::reference(), &root).unwrap();
            let keys = current.members().map(|member| reader.field(&member, NodeKey::reference()).unwrap())
                .collect::<BTreeSet<_>>();
            assert_eq!(keys, BTreeSet::from([0, 1]));
            assert_eq!(current.identity(), prior.output().1.identity(), "retirement without a split preserves the persistent name");
            let pin = reader.runtime.read_truth().pin_read_capability(reader.snapshot, &[], reader.execution).unwrap();
            assert_ne!(pin.snapshot().version_id(), prior.output().1.aggregate().source().version_id());
            let equal = reader.execution.run_native_with_checkpoint(|context|
                prior.output().1.aggregate().remains_equal_at(&pin, reader.execution, context, None).unwrap()).unwrap();
            assert!(!equal.0, "actual changed-product membership cannot reuse the old component answer");
        }, Allocation::SystemAllocation).unwrap();
    }).unwrap();
    world.application.close_live_delivery();
}
