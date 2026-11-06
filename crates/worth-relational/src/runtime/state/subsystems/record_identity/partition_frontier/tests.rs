use super::*;
use crate::history::data::RecordAllocationClass as Class;

#[test]
fn partition_high_water_includes_both_record_classes_pending_and_burned_namespaces() {
    let owner = RecordIdentitySubsystem::default();
    let serial = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(1024 * 1024)),
        worth_execution::CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&serial);
    assert_eq!(owner.observe_partition_high_water(&request).unwrap(), None);
    let entity = owner.reserve(Class::Entity, PartitionId(51)).unwrap();
    let relation = owner.reserve(Class::Relation, PartitionId(3)).unwrap();
    assert_eq!(
        owner.observe_partition_high_water(&request).unwrap(),
        Some(PartitionId(51)),
        "lexicographic class order cannot hide the larger entity partition"
    );
    let sibling = owner.clone();
    let pending = sibling.reserve(Class::Relation, PartitionId(72)).unwrap();
    assert_eq!(
        owner.observe_partition_high_water(&request).unwrap(),
        Some(PartitionId(72))
    );
    drop((pending, entity, relation));
    assert_eq!(
        owner.observe_partition_high_water(&request).unwrap(),
        Some(PartitionId(72)),
        "abandonment burns slots without erasing their namespace"
    );
    let restored = RecordIdentitySubsystem::default();
    for row in owner.frontier_snapshot() {
        restored.restore_frontier(row.0, row.1, row.2);
    }
    assert_eq!(
        restored.observe_partition_high_water(&request).unwrap(),
        Some(PartitionId(72)),
        "the existing restored allocator frontier needs no record bodies"
    );
}
