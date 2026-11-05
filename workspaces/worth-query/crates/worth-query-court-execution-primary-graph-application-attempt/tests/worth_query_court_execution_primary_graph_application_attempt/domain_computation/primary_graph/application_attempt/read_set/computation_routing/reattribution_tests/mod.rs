//! Pure metadata replacement; native completed-run closure is proved by the consumer.
use crate::domain_computation::primary_graph::application_attempt::read_set::computation_routing::tests::entity;
use crate::domain_computation::primary_graph::application_attempt::read_set::computation_routing::tests::key;
use crate::domain_computation::primary_graph::application_attempt::read_set::computation_routing::tests::partition;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::ComputationFactAttribution;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::vocabulary::application_attempt::computation_routing::ComputationRead;
use worth_foundational::facade::PartitionIdentity;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::read_set::computation_routing::SealedComputationFacts;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::retained_decision_facts::control::StorageControl;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::fact::WorthQueryApplicationObservedFact;
use worth_relational::facade::identity::KindId;
pub fn sealed() -> SealedComputationFacts {
    let mut reads = ComputationFactAttribution::default();
    reads.record(key(1), partition(7));
    reads.record(key(1), ComputationRead::Membership);
    reads.record(key(2), partition(9));
    let facts = (1..=2)
        .map(|n| {
            (
                key(n),
                WorthQueryApplicationObservedFact::Entity {
                    entity_id: entity(n),
                    kind: KindId::new(1),
                },
            )
        })
        .collect();
    SealedComputationFacts::at_seal_for_test(reads, &facts)
}
pub fn control() -> StorageControl<'static, 'static> {
    StorageControl::new_for_test(
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        None,
    )
}
#[test]
fn moved_readers_replace_old_edges_preserving_shared_roles_original_ordinals_and_history() {
    let prior = sealed();
    let (one, two) = (key(1), key(2));
    let calls = BTreeSet::from([partition(7), partition(9)]);
    let fresh = BTreeMap::from([(partition(7), vec![&two]), (partition(9), vec![&one])]);
    let next = prior
        .reattribute_for_test(&calls, &fresh, control())
        .unwrap()
        .unwrap();
    assert_eq!(
        (*(*next.metadata_for_test())[&one].readers_for_test())
            .reads()
            .collect::<Vec<_>>(),
        [ComputationRead::Membership, partition(9)]
    );
    assert_eq!(
        (*(*next.metadata_for_test())[&two].readers_for_test())
            .reads()
            .collect::<Vec<_>>(),
        [partition(7)]
    );
    assert_eq!(
        (*(*prior.metadata_for_test())[&one].readers_for_test()).partitions_for_test(),
        [PartitionIdentity::new(7)]
    );
    assert_eq!(
        (*next.index_for_test())
            .keys_for_test(partition(7))
            .collect::<Vec<_>>(),
        [&two]
    );
    assert_eq!(
        (*next.index_for_test())
            .keys_for_test(partition(9))
            .collect::<Vec<_>>(),
        [&one]
    );
    assert!(next.original.same_backing_for_test(&prior.original));
    assert_eq!((*next.layout_for_test()), (*prior.layout_for_test()));
    assert_eq!(
        (*(*next.metadata_for_test())[&one].ordinal_for_test()),
        (*(*prior.metadata_for_test())[&one].ordinal_for_test())
    );
}
#[test]
fn same_count_substitution_and_lost_last_reader_require_the_full_path() {
    let prior = sealed();
    let other = key(3);
    let one = key(1);
    let calls = BTreeSet::from([partition(9)]);
    let substituted = BTreeMap::from([(partition(9), vec![&other])]);
    assert!(prior
        .reattribute_for_test(&calls, &substituted, control())
        .unwrap()
        .is_none());
    let removed = BTreeMap::from([(partition(9), vec![&one])]);
    assert!(prior
        .reattribute_for_test(&calls, &removed, control())
        .unwrap()
        .is_none());
    assert_eq!(
        (*prior.index_for_test())
            .keys_for_test(partition(9))
            .collect::<Vec<_>>(),
        [&key(2)]
    );
}

#[test]
fn retired_and_inserted_calls_transfer_exact_keys_without_replacing_original_ordinals() {
    let prior = sealed();
    let (one, two) = (key(1), key(2));
    let calls = BTreeSet::from([partition(7), partition(9), partition(11)]);
    let fresh = BTreeMap::from([(partition(7), vec![&one]), (partition(11), vec![&two])]);
    let next = prior
        .reattribute_for_test(&calls, &fresh, control())
        .unwrap()
        .unwrap();
    assert!((*next.index_for_test())
        .keys_for_test(partition(9))
        .next()
        .is_none());
    assert!(!(*(*next.index_for_test()).calls_for_test()).contains_key(&partition(9)));
    assert_eq!(
        (*next.index_for_test())
            .keys_for_test(partition(11))
            .collect::<Vec<_>>(),
        [&two]
    );
    for key in [&one, &two] {
        assert_eq!(
            (*(*next.metadata_for_test())[key].ordinal_for_test()),
            (*(*prior.metadata_for_test())[key].ordinal_for_test())
        );
    }
    assert_eq!(
        (*(*next.metadata_for_test())[&two].readers_for_test()).partitions_for_test(),
        [PartitionIdentity::new(11)]
    );
    assert!(next.original.same_backing_for_test(&prior.original));
    assert_eq!(
        (*prior.index_for_test())
            .keys_for_test(partition(9))
            .collect::<Vec<_>>(),
        [&two]
    );
    let wrong = key(3);
    let substituted = BTreeMap::from([(partition(7), vec![&one]), (partition(11), vec![&wrong])]);
    assert!(prior
        .reattribute_for_test(&calls, &substituted, control())
        .unwrap()
        .is_none());
}
