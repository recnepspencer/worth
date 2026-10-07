//! A fresh keyed route declares one reroute and one membership visit.
use worth_execution::{KeyedItem, KeyedPartitioner, PartitionItemId, PartitionWork};
use worth_foundational::PartitionIdentity;
#[test]
fn fresh_keyed_route_charges_one_reroute_and_one_membership() {
    let declared = PartitionWork {
        items_rerouted: 1,
        members_visited: 1,
        ..PartitionWork::default()
    };
    let mut routes = KeyedPartitioner::new();
    let charged = routes
        .upsert(
            KeyedItem {
                item: PartitionItemId(1),
                key: 7_u64,
                partition: PartitionIdentity::new(7),
            },
            |_| Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(charged, declared);
    // One route plus its one membership: hand-counted, independent of units().
    assert_eq!(charged.units(), Some(2));
    assert_eq!(declared.units(), Some(2));
}
