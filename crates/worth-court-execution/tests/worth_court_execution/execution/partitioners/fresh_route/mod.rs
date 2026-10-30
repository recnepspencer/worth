// A fresh keyed route declares one reroute and one membership visit.
pub(crate) use {
    worth_execution::KeyedItem, worth_execution::PartitionItemId, worth_execution::PartitionWork,
};
#[test]
fn fresh_keyed_route_charges_one_reroute_and_one_membership() {
    let declared = PartitionWork {
        items_rerouted: 1,
        members_visited: 1,
        ..worth_execution::PartitionWork::default()
    };
    let mut routes = worth_execution::KeyedPartitioner::new();
    let charged = routes
        .upsert(
            KeyedItem {
                item: PartitionItemId(1),
                key: 7_u64,
                partition: worth_foundational::PartitionIdentity::new(7),
            },
            |_| Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(charged, declared);
    // One route plus its one membership: hand-counted, independent of units().
    assert_eq!(charged.units(), Some(2));
    assert_eq!(declared.units(), Some(2));
}
