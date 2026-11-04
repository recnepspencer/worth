//! Original Native bodies and actual selected-version retirements, including twins.

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::decode as decode_producer_facts;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::encode as encode_producer_facts;
pub use std::collections::BTreeSet;
pub use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
pub use worth_relational::facade::mvcc::CompanionPreflightStop;
pub use worth_relational::facade::identity::EntityId;
pub use worth_execution::ExecutionRequest;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact as Fact;
pub use worth_relational::facade::runtime::RelationalRuntime;
pub use worth_relational::facade::snapshots::SnapshotHandle;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::structural_walk::StructuralWalkObservation;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::retirement::structural::prepare;
pub use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::retirement::structural::tests;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::aggregate_observation::AggregateObservation;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::aggregate_observation::AggregateQuery;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::movement::Movement;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::structural_walk::read::StructuralWalkQuery;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::schema::*;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::world::World;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture;
pub use worth_execution::CancellationSource;
pub use worth_execution::ExecutionAllocationPolicy as Allocation;
pub use worth_execution::SerialMemoryBudget;
pub use worth_execution::SerialRequest;
pub use worth_relational::facade::identity::KindId;
pub use worth_relational::facade::symbols::ClientKey;
pub use worth_relational::facade::transactions::AspectFieldPatch;
pub use worth_relational::facade::transactions::CreateIntent;
pub use worth_relational::facade::transactions::DeleteEntityIntent;
pub use worth_relational::facade::transactions::DeleteRelationIntent;
pub use worth_relational::facade::transactions::EntityMutationIntent;
pub use worth_relational::facade::transactions::EntityReference;
pub use worth_relational::facade::transactions::MutationIntent;
pub use worth_relational::facade::transactions::RelationMutationIntent;
pub use worth_relational::facade::transactions::RelationSpec;
pub use worth_relational::facade::transactions::WorkerIntentBatch;

pub use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::evidence_seal::postcommit_currentness::retirement::structural::tests::orbit_world::*;

#[test]
fn whole_structural_retirement_requires_complete_correspondence_and_keeps_external_reads() {
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let owned = capture(&world, 0, &execution);
    let external = capture(&world, 3, &execution);
    let retired = nodes(&owned);
    let foreign_world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
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
            .find(|entity| **entity != observation.key_for_test().member())
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
            assert_eq!(old.locator_identity_for_test(), new.locator_identity_for_test());
            assert!(matches!(new, Fact::RetiredOutputEntity { .. }));
            assert_eq!(
                new.source_currentness_in_for_test(runtime, &execution, snapshot)
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
                fact.source_currentness_in_for_test(runtime, &execution, snapshot)
                    .unwrap()
                    .unwrap()
                    .movement(),
                Movement::Unmoved
            );
        }
        rebased
    });
    let encoded = encode_producer_facts(&rebased).unwrap();
    let restored = decode_producer_facts(&encoded, None, Allocation::SystemAllocation).unwrap();
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::sequence::FactSequence;
    assert_eq!(restored.iter().cloned().collect::<Vec<_>>(), rebased);
    let edge = walk(&external).native_for_test().relations()[0];
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation_on_application(
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
                .source_currentness_in_for_test(runtime, &execution, snapshot)
                .unwrap()
                .unwrap()
                .movement(),
            Movement::Moved
        );
        for fact in &rebased {
            assert_eq!(
                fact.source_currentness_in_for_test(runtime, &execution, snapshot)
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
    let _isolation = worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::reconstruction_execution::isolated_request_owner();
    let world = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::application_attempt::invariant_orbits::decision::identity::install();
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let candidates: Vec<_> = (0..3)
        .map(|key| (key, capture(&world, key, &execution)))
        .collect();
    let Fact::Aggregate { observation } = &candidates[0].1[0] else {
        unreachable!()
    };
    let anchor = observation.native().anchor();
    let member = |facts: &[Fact]| match &facts[0] {
        Fact::Aggregate { observation } => observation.key_for_test().member(),
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
    let incoming = walk(facts)
        .native_for_test()
        .relations()
        .iter()
        .find(|edge| edge.target == anchor)
        .unwrap();
    let outgoing = walk(facts)
        .native_for_test()
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
    worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::product_publication::publish_relational_mutation_on_application(&execution, &world.application, batch);
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
            prepare(runtime, &stopped, snapshot, &facts[0], &nodes(facts)),
            Err(CompanionPreflightStop::Execution(Cause::Cancelled))
        ));
    });
}

#[path = "orbit_world/mod.rs"]
pub mod orbit_world;
