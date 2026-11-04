use std::collections::BTreeMap;

use worth_foundational::facade::PartitionIdentity;
use worth_relational::facade::identity::EntityId;
use worth_relational::facade::identity::KindId;
use worth_relational::facade::identity::PartitionId;

use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationFactAttribution;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationFactReaders;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationRead;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::SealedComputationFacts;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationFactKey;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;

pub fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

pub fn key(slot: u64) -> WorthQueryApplicationFactKey {
    WorthQueryApplicationFactKey::Entity {
        entity: "entry".to_owned(),
        entity_id: entity(slot),
    }
}

pub fn partition(identity: u64) -> ComputationRead {
    ComputationRead::Partition(PartitionIdentity::new(identity))
}

pub fn item(identity: u64) -> ComputationRead {
    ComputationRead::ItemKey(worth_execution::PartitionItemId(identity))
}

pub fn partitions(identities: &[u64]) -> Vec<PartitionIdentity> {
    identities
        .iter()
        .copied()
        .map(PartitionIdentity::new)
        .collect()
}

#[test]
fn every_consuming_partition_survives_in_the_facts_reverse_index() {
    let mut attribution = ComputationFactAttribution::default();
    attribution.record(key(1), partition(9));
    attribution.record(key(1), partition(7));
    let sealed = readers(attribution, 1);
    assert_eq!(
        sealed[0].1.partitions_for_test(),
        partitions(&[7, 9]),
        "a consumed fact's reverse index must retain every consuming partition"
    );
    let adjacency = |limit| {
        WorthQueryApplicationFactKey::Adjacency {
        relation: "owner".into(),
        anchor: entity(1),
        direction: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationAdjacencyDirection::Incoming,
        maximum_work_units: limit,
    }
    };
    let mut attribution = ComputationFactAttribution::default();
    attribution.record(adjacency(2), ComputationRead::Membership);
    attribution.record(adjacency(5), item(11));
    attribution.record(adjacency(9), partition(7));
    attribution.record(adjacency(3), partition(9));
    let mut handler = std::collections::BTreeSet::from([adjacency(4)]);
    let host = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::serial_request();
    let execution = worth_execution::ExecutionRequest::serial(&host);
    attribution
        .normalize_adjacencies_for_test(&mut handler, &execution)
        .unwrap();
    assert_eq!(handler, std::collections::BTreeSet::from([adjacency(9)]));
    let observed = BTreeMap::from([(
        adjacency(9),
        WorthQueryApplicationObservedFact::Adjacency {
            relation_kind: KindId::new(3),
            anchor: entity(1),
            direction: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationAdjacencyDirection::Incoming,
            maximum_work_units: 9,
            relations: Vec::new(),
        },
    )]);
    let sealed = SealedComputationFacts::at_seal_for_test(attribution, &observed);
    let (_, _, readers) = sealed.facts_for_test().next().unwrap();
    assert_eq!(
        readers.reads().collect::<Vec<_>>(),
        [
            ComputationRead::Membership,
            item(11),
            partition(7),
            partition(9),
        ]
    );
}

/// Seals the facts over the keys `1..=count`, each observed as an entity of
/// its own kind, and returns what each computation fact kept: its readers, in
/// key order, after checking it kept the content seal observed.
pub fn readers(
    attribution: ComputationFactAttribution,
    count: u32,
) -> Vec<(WorthQueryApplicationFactKey, ComputationFactReaders)> {
    let observed = (1..=count)
        .map(|slot: u32| {
            let fact = WorthQueryApplicationObservedFact::Entity {
                entity_id: entity(slot.into()),
                kind: KindId::new(slot),
            };
            (key(slot.into()), fact)
        })
        .collect::<BTreeMap<_, _>>();
    SealedComputationFacts::at_seal_for_test(attribution, &observed)
        .facts_for_test()
        .map(|(key, fact, readers)| {
            assert_eq!(
                fact, &observed[key],
                "a fact keeps the content seal observed"
            );
            (key.clone(), readers.clone())
        })
        .collect()
}

#[test]
fn every_fact_keeps_the_calls_that_read_it_and_its_sealed_content() {
    let mut attribution = ComputationFactAttribution::default();
    // Fact 1: one partition. Fact 2: two partitions, the greater first.
    attribution.record(key(1), partition(7));
    attribution.record(key(2), partition(9));
    attribution.record(key(2), partition(7));
    // Fact 3: the membership. Fact 4: the keys of two items.
    attribution.record(key(3), ComputationRead::Membership);
    attribution.record(key(4), item(12));
    attribution.record(key(4), item(11));
    // Fact 5: nothing of the computation, so it is not the computation's.

    assert_eq!(
        readers(attribution, 5),
        [
            (
                key(1),
                ComputationFactReaders::read_by_for_test(false, [], partitions(&[7]))
            ),
            (
                key(2),
                ComputationFactReaders::read_by_for_test(false, [], partitions(&[7, 9]))
            ),
            (key(3), ComputationFactReaders::read_by_for_test(true, [], [])),
            (key(4), ComputationFactReaders::read_by_for_test(false, [11, 12], [])),
        ]
    );
}

#[test]
fn a_key_read_again_by_the_same_call_names_the_call_once() {
    let mut attribution = ComputationFactAttribution::default();
    attribution.record(key(1), partition(7));
    attribution.record(key(1), partition(7));
    attribution.record(key(2), item(3));
    attribution.record(key(2), item(3));

    assert_eq!(
        readers(attribution, 2),
        [
            (
                key(1),
                ComputationFactReaders::read_by_for_test(false, [], partitions(&[7]))
            ),
            (key(2), ComputationFactReaders::read_by_for_test(false, [3], [])),
        ]
    );
}

#[test]
fn a_key_read_in_several_classes_belongs_to_every_one_of_them() {
    // Each pair of classes reads one key, once in each order of arrival, and
    // key 7 is read in all three.
    let mut attribution = ComputationFactAttribution::default();
    attribution.record(key(1), partition(7));
    attribution.record(key(1), ComputationRead::Membership);
    attribution.record(key(2), ComputationRead::Membership);
    attribution.record(key(2), partition(7));
    attribution.record(key(3), partition(7));
    attribution.record(key(3), item(4));
    attribution.record(key(4), item(4));
    attribution.record(key(4), partition(7));
    attribution.record(key(5), item(4));
    attribution.record(key(5), ComputationRead::Membership);
    attribution.record(key(6), ComputationRead::Membership);
    attribution.record(key(6), item(4));
    attribution.record(key(7), partition(9));
    attribution.record(key(7), item(5));
    attribution.record(key(7), ComputationRead::Membership);

    let membership_and_partition = ComputationFactReaders::read_by_for_test(true, [], partitions(&[7]));
    let item_and_partition = ComputationFactReaders::read_by_for_test(false, [4], partitions(&[7]));
    let membership_and_item = ComputationFactReaders::read_by_for_test(true, [4], []);
    assert_eq!(
        readers(attribution, 7),
        [
            (key(1), membership_and_partition.clone()),
            (key(2), membership_and_partition),
            (key(3), item_and_partition.clone()),
            (key(4), item_and_partition),
            (key(5), membership_and_item.clone()),
            (key(6), membership_and_item),
            (
                key(7),
                ComputationFactReaders::read_by_for_test(true, [5], partitions(&[9]))
            ),
        ]
    );
}

#[test]
fn ordinal_alignment_includes_handler_gaps_and_retains_shared_call_keys() {
    let observed = (1..=5)
        .map(|slot| {
            (
                key(slot),
                WorthQueryApplicationObservedFact::Entity {
                    entity_id: entity(slot),
                    kind: KindId::new(slot as u32),
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut reads = ComputationFactAttribution::default();
    reads.record(key(2), partition(7));
    reads.record(key(2), partition(9));
    reads.record(key(4), ComputationRead::Membership);
    let sealed = SealedComputationFacts::at_seal_for_test(reads, &observed);

    assert!(
        sealed.at_ordinal_for_test(0).is_none(),
        "a handler-only ordinal is not a computation fact"
    );
    assert_eq!(sealed.at_ordinal_for_test(1).unwrap().0, &key(2));
    assert!(sealed.at_ordinal_for_test(2).is_none());
    assert_eq!(sealed.at_ordinal_for_test(3).unwrap().0, &key(4));
    assert!(sealed.at_ordinal_for_test(4).is_none());
    assert_eq!(
        sealed.call_index().keys_for_test(partition(7)).collect::<Vec<_>>(),
        [&key(2)]
    );
    assert_eq!(
        sealed.call_index().keys_for_test(partition(9)).collect::<Vec<_>>(),
        [&key(2)]
    );
    assert_eq!(sealed.observation_work_bound(), Some(2));
}

#[test]
fn completion_metadata_retains_the_exact_original_pre_effect_payload() {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::FactPayload;
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_checkpoint::facts::payload::native::NativeFactPayloadBuilder;
    let observed: BTreeMap<_, _> = (1..=5)
        .map(|slot| {
            (
                key(slot),
                WorthQueryApplicationObservedFact::Entity {
                    entity_id: entity(slot),
                    kind: KindId::new(slot as u32),
                },
            )
        })
        .collect();
    let mut reads = ComputationFactAttribution::default();
    reads.record(key(2), partition(7));
    reads.record(key(2), partition(9));
    reads.record(key(4), ComputationRead::Membership);
    let metadata = SealedComputationFacts::prepare(
        reads,
        observed.keys(),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
    .unwrap();
    let mut body = NativeFactPayloadBuilder::allocate_completed_read_for_test(
        observed.len(),
        StorageControl::new_for_test(
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            None,
        ),
    )
    .unwrap();
    for fact in observed.values().cloned() {
        body.push(fact).unwrap();
    }
    let body: FactPayload = body.seal().unwrap();
    let sealed = metadata.bind_fixture_for_test(body.clone());
    assert!(std::ptr::eq(
        body.get(1).unwrap(),
        sealed.fact_for_test(&key(2)).unwrap()
    ));
    assert!(std::ptr::eq(
        body.get(3).unwrap(),
        sealed.at_ordinal_for_test(3).unwrap().1
    ));
    assert!(sealed.at_ordinal_for_test(0).is_none(), "handler gap remains exact");
    let cloned = sealed.clone();
    assert!(std::sync::Arc::ptr_eq(&(*sealed.index_for_test()), &(*cloned.index_for_test())));
    assert!(std::sync::Arc::ptr_eq(&(*sealed.metadata_for_test()), &(*cloned.metadata_for_test())));
    assert!(sealed.original.same_backing_for_test(&cloned.original));
    let replacement = WorthQueryApplicationObservedFact::Entity {
        entity_id: entity(2),
        kind: KindId::new(99),
    };
    let mut adversarial = cloned;
    adversarial.replace_fact_for_test(&key(2), replacement.clone());
    assert_eq!(adversarial.fact_for_test(&key(2)), Some(&replacement));
    assert_eq!(
        sealed.fact_for_test(&key(2)),
        body.get(1),
        "another observer cannot alter original comparator facts"
    );
}

#[test]
fn native_fact_index_preserves_ordinals_and_readers_without_an_observation_bill() {
    use worth_execution::CancellationToken;
    use worth_execution::ExecutionRequest;
    use worth_execution::SerialRequest;
    use worth_foundational::DeterminismContract;
    use worth_foundational::ExecutionPosture;
    use worth_foundational::ExecutionRequestPolicy;
    let policy = ExecutionRequestPolicy::for_workers(
        ExecutionPosture::Serial,
        DeterminismContract::CanonicalBitwise,
        std::num::NonZeroUsize::MIN,
    );
    let request = SerialRequest::from_policy(&policy, CancellationToken::new(), None);
    let (sealed, _) = ExecutionRequest::serial(&request)
        .run_native(|_| {
            let observed = BTreeMap::from([(
                key(1),
                WorthQueryApplicationObservedFact::Entity {
                    entity_id: entity(1),
                    kind: KindId::new(1),
                },
            )]);
            let mut reads = ComputationFactAttribution::default();
            reads.record(key(1), partition(7));
            SealedComputationFacts::at_seal_for_test(reads, &observed)
        })
        .unwrap();
    assert_eq!(sealed.observation_work_bound(), None);
    assert_eq!(sealed.at_ordinal_for_test(0).unwrap().0, &key(1));
    assert_eq!(
        sealed.call_index().keys_for_test(partition(7)).collect::<Vec<_>>(),
        [&key(1)]
    );
}
