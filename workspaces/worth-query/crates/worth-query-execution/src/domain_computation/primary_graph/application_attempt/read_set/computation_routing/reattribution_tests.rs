//! Pure metadata replacement; native completed-run closure is proved by the consumer.
use super::tests::{entity, key, partition};
use super::*;
use crate::domain_computation::primary_graph::application_attempt::{
    retained_decision_facts::StorageControl, WorthQueryApplicationObservedFact,
};
use worth_relational::facade::identity::KindId;
fn sealed() -> SealedComputationFacts {
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
    SealedComputationFacts::at_seal(reads, &facts)
}
fn control() -> StorageControl<'static, 'static> {
    StorageControl::new(
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
        .reattribute(&calls, &fresh, control())
        .unwrap()
        .unwrap();
    assert_eq!(
        next.metadata[&one].readers.reads().collect::<Vec<_>>(),
        [ComputationRead::Membership, partition(9)]
    );
    assert_eq!(
        next.metadata[&two].readers.reads().collect::<Vec<_>>(),
        [partition(7)]
    );
    assert_eq!(
        prior.metadata[&one].readers.partitions(),
        [PartitionIdentity::new(7)]
    );
    assert_eq!(next.index.keys(partition(7)).collect::<Vec<_>>(), [&two]);
    assert_eq!(next.index.keys(partition(9)).collect::<Vec<_>>(), [&one]);
    assert!(next.original.same_backing(&prior.original));
    assert_eq!(next.layout, prior.layout);
    assert_eq!(next.metadata[&one].ordinal, prior.metadata[&one].ordinal);
}
#[test]
fn same_count_substitution_and_lost_last_reader_require_the_full_path() {
    let prior = sealed();
    let other = key(3);
    let one = key(1);
    let calls = BTreeSet::from([partition(9)]);
    let substituted = BTreeMap::from([(partition(9), vec![&other])]);
    assert!(prior
        .reattribute(&calls, &substituted, control())
        .unwrap()
        .is_none());
    let removed = BTreeMap::from([(partition(9), vec![&one])]);
    assert!(prior
        .reattribute(&calls, &removed, control())
        .unwrap()
        .is_none());
    assert_eq!(
        prior.index.keys(partition(9)).collect::<Vec<_>>(),
        [&key(2)]
    );
}
