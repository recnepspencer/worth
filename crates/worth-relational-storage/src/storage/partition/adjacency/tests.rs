use super::*;
use crate::identity::data::PartitionId;

#[test]
fn kind_buckets_isolate_current_work_and_retain_historical_membership() {
    let policy = AdjacencyPolicy {
        backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
        small_degree_inline_capacity: 4,
    };
    let first = RelationId::new(PartitionId::main(), 1, 1);
    let unrelated = RelationId::new(PartitionId::main(), 2, 1);
    let mut adjacency = AdjacencySet::new(&policy);
    adjacency.insert(KindId(7), first);
    adjacency.insert(KindId(8), unrelated);

    assert_eq!(adjacency.current_kind_ids(KindId(7)), [first]);
    assert_eq!(adjacency.current_kind_ids(KindId(8)), [unrelated]);

    adjacency.remove(KindId(7), &first);
    assert!(adjacency.current_kind_ids(KindId(7)).is_empty());
    assert_eq!(adjacency.historical_kind_ids(KindId(7)), [first]);
}
