//! Authentic reservation handles, rather than copied canonical data, authorize reapply.
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
    pending.prepare_reapplication().unwrap();
    (authority, pending, record)
}

#[test]
fn owned_prefix_preserves_reclaimed_generation_and_can_append_later_consequences() {
    let (authority, mut pending, record) = reserved_reclaimed();
    let before = pending.canonical().to_vec();
    let live = authority.pending_snapshot();
    let applied = pending
        .reserve(RecordAllocationClass::Entity, PartitionId(7))
        .unwrap();
    assert_eq!((applied.slot, applied.generation), (2, 5));
    pending.record(record);
    pending.finish_reapplication().unwrap();
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
fn wrong_class_partition_extra_and_missing_requests_refuse_before_custody_can_escape() {
    for (class, partition) in [
        (RecordAllocationClass::Relation, PartitionId(7)),
        (RecordAllocationClass::Entity, PartitionId(8)),
    ] {
        let (authority, mut pending, _) = reserved_reclaimed();
        assert!(matches!(
            pending.reserve(class, partition),
            Err(RecordAllocationDenial::ReplayTargetMismatch { .. })
        ));
        assert_eq!(pending.reservation_count(), 1);
        drop(pending);
        assert!(authority.pending_snapshot().is_empty());
        assert_eq!(authority.reusable_snapshot().len(), 1);
    }
    let (authority, mut extra, record) = reserved_reclaimed();
    extra
        .reserve(RecordAllocationClass::Entity, PartitionId(7))
        .unwrap();
    extra.record(record);
    assert!(matches!(
        extra.reserve(RecordAllocationClass::Entity, PartitionId(7)),
        Err(RecordAllocationDenial::ReplayEvidenceMissing { ordinal: 1 })
    ));
    drop(extra);
    assert!(authority.pending_snapshot().is_empty());
    let (authority, mut missing, _) = reserved_reclaimed();
    assert_eq!(
        missing.finish_reapplication(),
        Err(RecordAllocationDenial::ReplayEvidenceRemaining { remaining: 1 })
    );
    drop(missing);
    assert!(authority.pending_snapshot().is_empty());
}

#[test]
fn copied_ordinal_slot_or_generation_cannot_replace_the_owned_reservation() {
    for (case, counterfeit) in [
        CanonicalRecordAllocation::with_origin(
            9,
            RecordRef::Entity(EntityId::new(PartitionId(7), 2, 5)),
            RecordAllocationOrigin::Reclaimed {
                prior_generation: 4,
            },
        ),
        CanonicalRecordAllocation::with_origin(
            0,
            RecordRef::Entity(EntityId::new(PartitionId(7), 3, 5)),
            RecordAllocationOrigin::Reclaimed {
                prior_generation: 4,
            },
        ),
        CanonicalRecordAllocation::with_origin(
            0,
            RecordRef::Entity(EntityId::new(PartitionId(7), 2, 6)),
            RecordAllocationOrigin::Reclaimed {
                prior_generation: 5,
            },
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let (authority, mut pending, _) = reserved_reclaimed();
        pending.canonical[0] = counterfeit;
        let denial = pending
            .reserve(RecordAllocationClass::Entity, PartitionId(7))
            .unwrap_err();
        let expected = match case {
            0 => RecordAllocationDenial::ReplayEvidenceUnexpected {
                expected_ordinal: 0,
                observed_ordinal: 9,
            },
            1 => RecordAllocationDenial::ReplayTargetMismatch {
                ordinal: 0,
                expected: RecordRef::Entity(EntityId::new(PartitionId(7), 3, 5)),
                class: RecordAllocationClass::Entity,
                partition_id: PartitionId(7),
            },
            2 => RecordAllocationDenial::ReplayGenerationMismatch {
                ordinal: 0,
                class: RecordAllocationClass::Entity,
                partition_id: PartitionId(7),
                slot: 2,
                expected_generation: 5,
                observed_generation: 6,
            },
            _ => unreachable!(),
        };
        assert_eq!(denial, expected);
        assert_eq!(authority.pending_snapshot().len(), 1);
        drop(pending);
        assert!(authority.pending_snapshot().is_empty());
        assert_eq!(authority.reusable_snapshot().len(), 1);
    }
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
