//! Original Native bodies and actual selected-version retirements, including twins.
use super::*;
use crate::domain_computation::primary_graph::{
    aggregate_observation::{AggregateObservation, AggregateQuery},
    application_attempt::Movement,
    structural_walk::StructuralWalkQuery,
    tests::{
        application_attempt::invariant_orbits::{decision::identity, schema::*, world::World},
        fixture,
    },
};
use worth_execution::{
    CancellationSource, ExecutionAllocationPolicy as Allocation, SerialMemoryBudget, SerialRequest,
};
use worth_relational::facade::{
    identity::KindId,
    symbols::ClientKey,
    transactions::{
        AspectFieldPatch, CreateIntent, DeleteEntityIntent, DeleteRelationIntent,
        EntityMutationIntent, EntityReference, MutationIntent, RelationMutationIntent,
        RelationSpec, WorkerIntentBatch,
    },
};

mod orbit_world;
use orbit_world::*;

#[test]
fn whole_structural_retirement_requires_complete_correspondence_and_keeps_external_reads() {
    let _isolation = fixture::isolated_request_owner();
    let world = identity::install();
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let owned = capture(&world, 0, &execution);
    let external = capture(&world, 3, &execution);
    let retired = nodes(&owned);
    let foreign_world = identity::install();
    let foreign = capture(&foreign_world, 0, &execution);
    assert_eq!(
        nodes(&foreign),
        retired,
        "the foreign fixture deliberately shares numeric IDs"
    );
    at_current(&world, &execution, |runtime, snapshot| {
        for fact in &owned {
            assert!(
                prepare(runtime, &execution, snapshot, fact, &retired)
                    .unwrap()
                    .is_none(),
                "a live record cannot supply retirement proof"
            );
        }
    });
    delete_group(&world, &execution, &owned);
    let mut missing = retired.clone();
    let Fact::Aggregate { observation } = &owned[0] else {
        unreachable!()
    };
    missing.remove(
        retired
            .iter()
            .find(|entity| **entity != observation.key().member())
            .unwrap(),
    );
    let rebased = at_current(&world, &execution, |runtime, snapshot| {
        for fact in &owned {
            assert!(prepare(runtime, &execution, snapshot, fact, &missing)
                .unwrap()
                .is_none());
            assert!(
                prepare(runtime, &execution, snapshot, fact, &BTreeSet::new())
                    .unwrap()
                    .is_none()
            );
        }
        for fact in &foreign {
            assert!(
                prepare(runtime, &execution, snapshot, fact, &retired)
                    .unwrap()
                    .is_none(),
                "coincident foreign-runtime IDs grant no retirement custody"
            );
        }
        let rebased: Vec<_> = owned
            .iter()
            .map(|fact| {
                prepare(runtime, &execution, snapshot, fact, &retired)
                    .unwrap()
                    .unwrap()
            })
            .collect();
        for (old, new) in owned.iter().zip(&rebased) {
            assert_eq!(old.locator_identity(), new.locator_identity());
            assert!(matches!(new, Fact::RetiredOutputEntity { .. }));
            assert_eq!(
                new.source_currentness_in(runtime, &execution, snapshot)
                    .unwrap()
                    .unwrap()
                    .movement(),
                Movement::Unmoved
            );
        }
        for fact in &external {
            assert!(prepare(runtime, &execution, snapshot, fact, &retired)
                .unwrap()
                .is_none());
            assert_eq!(
                fact.source_currentness_in(runtime, &execution, snapshot)
                    .unwrap()
                    .unwrap()
                    .movement(),
                Movement::Unmoved
            );
        }
        rebased
    });
    let encoded =
        crate::domain_computation::primary_graph::application_checkpoint::encode_producer_facts(
            &rebased,
        )
        .unwrap();
    let restored =
        crate::domain_computation::primary_graph::application_checkpoint::decode_producer_facts(
            &encoded,
            None,
            Allocation::SystemAllocation,
        )
        .unwrap();
    use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
    assert_eq!(restored.iter().cloned().collect::<Vec<_>>(), rebased);
    let edge = walk(&external).native().relations()[0];
    fixture::publish_relational_mutation_on_application(
        &execution,
        &world.application,
        WorkerIntentBatch::new("external-physical-successor-change")
            .push(MutationIntent::Relation(RelationMutationIntent::Delete(
                DeleteRelationIntent {
                    relation_id: edge.relation_id,
                },
            )))
            .push(replacement(edge.kind_id, edge.source, edge.target)),
    );
    at_current(&world, &execution, |runtime, snapshot| {
        assert_eq!(
            external[1]
                .source_currentness_in(runtime, &execution, snapshot)
                .unwrap()
                .unwrap()
                .movement(),
            Movement::Moved
        );
        for fact in &rebased {
            assert_eq!(
                fact.source_currentness_in(runtime, &execution, snapshot)
                    .unwrap()
                    .unwrap()
                    .movement(),
                Movement::Unmoved
            );
        }
    });
}

#[test]
fn retiring_the_selected_anchor_does_not_retire_the_surviving_group_read() {
    let _isolation = fixture::isolated_request_owner();
    let world = identity::install();
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let candidates: Vec<_> = (0..3)
        .map(|key| (key, capture(&world, key, &execution)))
        .collect();
    let Fact::Aggregate { observation } = &candidates[0].1[0] else {
        unreachable!()
    };
    let anchor = observation.native().anchor();
    let member = |facts: &[Fact]| match &facts[0] {
        Fact::Aggregate { observation } => observation.key().member(),
        _ => unreachable!(),
    };
    let facts = &candidates
        .iter()
        .find(|(_, facts)| member(facts) == anchor)
        .unwrap()
        .1;
    let survivor = candidates
        .iter()
        .find(|(_, facts)| member(facts) != anchor)
        .unwrap()
        .0;
    let Fact::Aggregate { observation } = &facts[0] else {
        unreachable!()
    };
    let original_identity = observation.native().identity();
    let incoming = walk(&facts)
        .native()
        .relations()
        .iter()
        .find(|edge| edge.target == anchor)
        .unwrap();
    let outgoing = walk(&facts)
        .native()
        .relations()
        .iter()
        .find(|edge| edge.source == anchor)
        .unwrap();
    let batch = WorkerIntentBatch::new("anchor-removal-surviving-group")
        .push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: incoming.relation_id,
            },
        )))
        .push(MutationIntent::Relation(RelationMutationIntent::Delete(
            DeleteRelationIntent {
                relation_id: outgoing.relation_id,
            },
        )))
        .push(replacement(
            incoming.kind_id,
            incoming.source,
            outgoing.target,
        ))
        .push(MutationIntent::Entity(EntityMutationIntent::Delete(
            DeleteEntityIntent { entity_id: anchor },
        )));
    fixture::publish_relational_mutation_on_application(&execution, &world.application, batch);
    let surviving = capture(&world, survivor, &execution);
    let Fact::Aggregate { observation } = &surviving[0] else {
        unreachable!()
    };
    assert_eq!(observation.native().identity(), original_identity);
    assert_eq!(observation.native().len(), 2);
    at_current(&world, &execution, |runtime, snapshot| {
        for fact in facts {
            assert!(prepare(
                runtime,
                &execution,
                snapshot,
                fact,
                &BTreeSet::from([anchor])
            )
            .unwrap()
            .is_none());
        }
    });
    let cancellation = CancellationSource::new();
    let original =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let stopped = ExecutionRequest::serial(&original);
    cancellation.cancel();
    at_current(&world, &execution, |runtime, snapshot| {
        assert!(matches!(
            prepare(runtime, &stopped, snapshot, &facts[0], &nodes(&facts)),
            Err(CompanionPreflightStop::Execution(Cause::Cancelled))
        ));
    });
}
