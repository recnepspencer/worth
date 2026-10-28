//! One carried allocation batch retains business identities and appends consequences.
use super::*;
use crate::identity::data::EntityId;

fn reserved_reclaimed() -> (RecordIdentitySubsystem, PendingRecordAllocations, RecordRef) {
    let authority = RecordIdentitySubsystem::default();
    let partition = PartitionId(7);
    authority.restore_generation(RecordAllocationClass::Entity, partition, 2, 4);
    authority.restore_reusable(RecordAllocationClass::Entity, partition, 2);
    let mut pending = authority.begin_allocations();
    let allocated = pending
        .reserve(RecordAllocationClass::Entity, partition)
        .unwrap();
    let record = RecordRef::Entity(EntityId::new(
        partition,
        allocated.slot as u64,
        allocated.generation,
    ));
    pending.record(record.clone());
    (authority, pending, record)
}

#[test]
fn owned_business_allocations_preserve_reclaimed_generation_and_append_consequences() {
    let (authority, mut pending, record) = reserved_reclaimed();
    let before = pending.canonical().to_vec();
    let live = authority.pending_snapshot();
    assert_eq!(
        record,
        RecordRef::Entity(EntityId::new(PartitionId(7), 2, 5))
    );
    assert_eq!(pending.canonical(), before);
    assert_eq!(authority.pending_snapshot(), live);
    assert_eq!(pending.reservation_count(), 1);
    let next = pending
        .reserve(RecordAllocationClass::Entity, PartitionId(7))
        .unwrap();
    pending.record(RecordRef::Entity(EntityId::new(
        PartitionId(7),
        next.slot as u64,
        next.generation,
    )));
    assert_eq!(pending.canonical().len(), 2);
    assert_eq!(pending.canonical()[1].ordinal(), 1);
    pending.finish_mutation().unwrap();
    pending.commit();
    assert!(authority.pending_snapshot().is_empty());
    assert!(authority.reusable_snapshot().is_empty());
}

#[test]
fn fork_releases_its_copied_reclaimed_reservation_without_releasing_source() {
    let (authority, pending, _) = reserved_reclaimed();
    let original = authority.pending_snapshot();
    let fork = crate::runtime::RuntimeSubsystem::fork(&authority);
    assert!(fork.pending_snapshot().is_empty());
    assert_eq!(fork.reusable_snapshot().len(), 1);
    assert_eq!(authority.pending_snapshot(), original);
    let mut fork_allocations = fork.begin_allocations();
    let next = fork_allocations
        .reserve(RecordAllocationClass::Entity, PartitionId(7))
        .unwrap();
    assert_eq!((next.slot, next.generation), (2, 6));
    assert_eq!(authority.pending_snapshot(), original);
    drop(pending);
    assert!(authority.pending_snapshot().is_empty());
}
