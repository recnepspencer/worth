use super::*;
use crate::identity::data::EntityId;
use crate::transactions::data::RecordRef;

#[test]
fn rejected_reclaimed_reservation_returns_only_its_lease() {
    let authority = RecordIdentitySubsystem::default();
    let partition_id = PartitionId(7);
    authority.restore_generation(RecordAllocationClass::Entity, partition_id, 2, 4);
    authority.restore_reusable(RecordAllocationClass::Entity, partition_id, 2);

    {
        let mut pending = PendingRecordAllocations::new(authority.clone(), None);
        let reserved = pending
            .reserve(RecordAllocationClass::Entity, partition_id)
            .unwrap();
        assert_eq!(reserved.slot, 2);
        assert_eq!(reserved.generation, 5);
    }

    assert_eq!(
        authority.reusable_snapshot(),
        vec![(RecordAllocationClass::Entity, partition_id, 2)]
    );
    assert_eq!(
        authority.generation_snapshot(),
        vec![(RecordAllocationClass::Entity, partition_id, 2, 5)]
    );
}

#[test]
fn rejected_append_reservation_burns_a_hole() {
    let authority = RecordIdentitySubsystem::default();
    let partition_id = PartitionId(7);
    {
        let mut pending = PendingRecordAllocations::new(authority.clone(), None);
        assert_eq!(
            pending
                .reserve(RecordAllocationClass::Entity, partition_id)
                .unwrap()
                .slot,
            0
        );
    }
    let mut next = PendingRecordAllocations::new(authority.clone(), None);
    assert_eq!(
        next.reserve(RecordAllocationClass::Entity, partition_id)
            .unwrap()
            .slot,
        1
    );
    assert!(authority.reusable_snapshot().is_empty());
}

#[test]
fn committed_reclaimed_reservation_is_consumed() {
    let authority = RecordIdentitySubsystem::default();
    let partition_id = PartitionId(7);
    authority.restore_generation(RecordAllocationClass::Entity, partition_id, 2, 4);
    authority.restore_reusable(RecordAllocationClass::Entity, partition_id, 2);
    let mut remaining = vec![
        (RecordAllocationClass::Entity, PartitionId(6), 0),
        (RecordAllocationClass::Entity, partition_id, 9),
        (RecordAllocationClass::Relation, partition_id, 0),
        (RecordAllocationClass::Entity, partition_id, 5),
        (RecordAllocationClass::Entity, PartitionId(8), 0),
    ];
    for &(class, partition, slot) in &remaining {
        authority.restore_reusable(class, partition, slot);
    }
    let mut pending = PendingRecordAllocations::new(authority.clone(), None);

    let selected = pending
        .reserve(RecordAllocationClass::Entity, partition_id)
        .unwrap();
    assert_eq!(
        selected.slot, 2,
        "lowest slot in the actual class and partition"
    );
    assert_eq!(selected.generation, 5);
    pending.commit();

    remaining.sort();
    assert_eq!(authority.reusable_snapshot(), remaining);
}

#[test]
fn replay_rejects_an_in_range_slot_without_reclamation_proof() {
    let authority = RecordIdentitySubsystem::default();
    let partition_id = PartitionId(7);
    authority.restore_generation(RecordAllocationClass::Entity, partition_id, 0, 1);
    let expected = CanonicalRecordAllocation::with_origin(
        0,
        RecordRef::Entity(EntityId::new(partition_id, 0, 2)),
        RecordAllocationOrigin::Reclaimed {
            prior_generation: 1,
        },
    );
    let mut pending = PendingRecordAllocations::new(authority, Some(vec![expected]));

    let denial = pending
        .reserve(RecordAllocationClass::Entity, partition_id)
        .unwrap_err();

    assert_eq!(
        denial,
        RecordAllocationDenial::ReplaySlotUnavailable {
            ordinal: 0,
            class: RecordAllocationClass::Entity,
            partition_id,
            slot: 0,
        }
    );
}

#[test]
fn sibling_branch_observations_share_one_append_frontier() {
    let authority = RecordIdentitySubsystem::default();
    let partition_id = PartitionId(11);

    let mut first = PendingRecordAllocations::new(authority.clone(), None);
    let first_record = first
        .reserve(RecordAllocationClass::Entity, partition_id)
        .unwrap();
    first.record(RecordRef::Entity(EntityId::new(
        partition_id,
        first_record.slot as u64,
        first_record.generation,
    )));
    first.commit();

    let mut sibling = PendingRecordAllocations::new(authority.clone(), None);
    let sibling_record = sibling
        .reserve(RecordAllocationClass::Entity, partition_id)
        .unwrap();

    assert_eq!(first_record.slot, 0);
    assert_eq!(sibling_record.slot, 1);
    assert_eq!(
        authority.frontier_snapshot(),
        vec![(RecordAllocationClass::Entity, partition_id, 2)]
    );
}

#[test]
fn divergent_branch_fanout_allocates_unique_slots_without_gap_reuse() {
    let authority = RecordIdentitySubsystem::default();
    let partition_id = PartitionId(19);
    let mut observed = std::collections::BTreeSet::new();

    for expected_slot in 0..4_096 {
        let mut branch = PendingRecordAllocations::new(authority.clone(), None);
        let reserved = branch
            .reserve(RecordAllocationClass::Entity, partition_id)
            .unwrap();
        assert_eq!(reserved.slot, expected_slot);
        assert!(observed.insert(reserved.slot));
        branch.commit();
    }

    assert_eq!(observed.len(), 4_096);
    assert_eq!(
        authority.frontier_snapshot(),
        vec![(RecordAllocationClass::Entity, partition_id, 4_096)]
    );
}
