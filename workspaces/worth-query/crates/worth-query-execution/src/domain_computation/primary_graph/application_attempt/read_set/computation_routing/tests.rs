use std::collections::BTreeSet;

use worth_execution::PartitionItemId;
use worth_foundational::facade::PartitionIdentity;
use worth_relational::facade::identity::{EntityId, PartitionId};

use super::{
    ComputationFactAttribution, ComputationFactReaders, ComputationFactRouting, ComputationRead,
};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationFactKey;

fn key(slot: u64) -> WorthQueryApplicationFactKey {
    WorthQueryApplicationFactKey::Entity {
        entity: "entry".to_owned(),
        entity_id: EntityId::new(PartitionId::main(), slot, 1),
    }
}

fn partition(identity: u64) -> ComputationRead {
    ComputationRead::Partition(PartitionIdentity::new(identity))
}

fn item(identity: u64) -> ComputationRead {
    ComputationRead::ItemKey(PartitionItemId(identity))
}

/// The table over the keys `1..=count`, which are the handler facts in order.
fn sealed(attribution: ComputationFactAttribution, count: u64) -> ComputationFactRouting {
    let keys = (1..=count).map(key).collect::<Vec<_>>();
    ComputationFactRouting::at_seal(attribution, keys.iter())
}

#[test]
fn every_fact_is_routed_to_the_calls_that_read_it() {
    let mut attribution = ComputationFactAttribution::default();
    // Fact 1: one partition. Fact 2: two partitions, the greater first.
    attribution.record(key(1), partition(7));
    attribution.record(key(2), partition(9));
    attribution.record(key(2), partition(7));
    // Fact 3: the membership. Fact 4: the keys of two items.
    attribution.record(key(3), ComputationRead::Membership);
    attribution.record(key(4), item(12));
    attribution.record(key(4), item(11));
    // Fact 5: nothing of the computation, so the handler's.

    let routing = sealed(attribution, 5);

    assert_eq!(
        routing.readers(),
        [
            ComputationFactReaders::Partitions(vec![PartitionIdentity::new(7)]),
            ComputationFactReaders::Partitions(vec![
                PartitionIdentity::new(7),
                PartitionIdentity::new(9)
            ]),
            ComputationFactReaders::Membership,
            ComputationFactReaders::ItemKeys(vec![PartitionItemId(11), PartitionItemId(12)]),
            ComputationFactReaders::Handler,
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
        sealed(attribution, 2).readers(),
        [
            ComputationFactReaders::Partitions(vec![PartitionIdentity::new(7)]),
            ComputationFactReaders::ItemKeys(vec![PartitionItemId(3)]),
        ]
    );
}

#[test]
fn a_key_read_in_two_classes_belongs_to_the_first_of_them() {
    // Each key is read in two classes, once in each order of arrival.
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
    // The handler read keys 7 and 8 itself, beside a partition and the
    // membership.
    attribution.record(key(7), partition(7));
    attribution.record(key(8), ComputationRead::Membership);
    attribution.yield_to_handler(&BTreeSet::from([key(7), key(8)]));

    assert_eq!(
        sealed(attribution, 8).readers(),
        [
            ComputationFactReaders::Membership,
            ComputationFactReaders::Membership,
            ComputationFactReaders::ItemKeys(vec![PartitionItemId(4)]),
            ComputationFactReaders::ItemKeys(vec![PartitionItemId(4)]),
            ComputationFactReaders::Membership,
            ComputationFactReaders::Membership,
            ComputationFactReaders::Handler,
            ComputationFactReaders::Handler,
        ]
    );
}
