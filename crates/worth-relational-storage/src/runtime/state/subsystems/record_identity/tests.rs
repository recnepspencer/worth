use super::*;
use crate::identity::data::EntityId;
use crate::transactions::data::primitives::RecordRef;

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

#[test]
fn staged_replay_allows_out_of_order_append_slots_within_one_commit_and_advances_frontier() {
    let authority = RecordIdentitySubsystem::default();
    let partition_id = PartitionId(23);
    let allocations = vec![
        CanonicalRecordAllocation::with_origin(
            0,
            RecordRef::Entity(EntityId::new(partition_id, 1, 1)),
            RecordAllocationOrigin::AppendFrontier,
        ),
        CanonicalRecordAllocation::with_origin(
            1,
            RecordRef::Entity(EntityId::new(partition_id, 0, 1)),
            RecordAllocationOrigin::AppendFrontier,
        ),
    ];
    authority
        .stage_replay_allocations_with_leading_gaps(allocations)
        .unwrap();
    let mut pending = authority.begin_allocations();
    let first = pending
        .reserve(RecordAllocationClass::Entity, partition_id)
        .unwrap();
    let second = pending
        .reserve(RecordAllocationClass::Entity, partition_id)
        .unwrap();
    assert_eq!((first.slot, first.generation), (1, 1));
    assert_eq!((second.slot, second.generation), (0, 1));
    pending.finish_mutation().unwrap();
    pending.commit();
    assert_eq!(
        authority.frontier_snapshot(),
        vec![(RecordAllocationClass::Entity, partition_id, 2)]
    );
}

#[test]
fn staged_replay_allows_restored_pending_append_reservation_below_frontier() {
    let authority = RecordIdentitySubsystem::default();
    let partition_id = PartitionId(31);
    authority.restore_frontier(RecordAllocationClass::Entity, partition_id, 2);
    authority.restore_generation(RecordAllocationClass::Entity, partition_id, 0, 1);
    authority
        .restore_pending(
            RecordAllocationClass::Entity,
            partition_id,
            0,
            1,
            RecordAllocationOrigin::AppendFrontier,
        )
        .unwrap();
    let allocations = vec![
        CanonicalRecordAllocation::with_origin(
            0,
            RecordRef::Entity(EntityId::new(partition_id, 0, 1)),
            RecordAllocationOrigin::AppendFrontier,
        ),
        CanonicalRecordAllocation::with_origin(
            1,
            RecordRef::Entity(EntityId::new(partition_id, 2, 1)),
            RecordAllocationOrigin::AppendFrontier,
        ),
    ];
    authority
        .stage_replay_allocations_with_leading_gaps(allocations)
        .unwrap();
    let mut pending = authority.begin_allocations();
    let restored = pending
        .reserve(RecordAllocationClass::Entity, partition_id)
        .unwrap();
    let fresh = pending
        .reserve(RecordAllocationClass::Entity, partition_id)
        .unwrap();
    assert_eq!((restored.slot, restored.generation), (0, 1));
    assert_eq!((fresh.slot, fresh.generation), (2, 1));
    pending.finish_mutation().unwrap();
    pending.commit();
    assert!(authority.pending_snapshot().is_empty());
    assert_eq!(
        authority.frontier_snapshot(),
        vec![(RecordAllocationClass::Entity, partition_id, 3)]
    );
}

#[test]
fn unstaged_out_of_order_append_slot_is_rejected_by_replay() {
    let authority = RecordIdentitySubsystem::default();
    let partition_id = PartitionId(29);
    let expected = CanonicalRecordAllocation::with_origin(
        0,
        RecordRef::Entity(EntityId::new(partition_id, 1, 1)),
        RecordAllocationOrigin::AppendFrontier,
    );
    let mut pending = PendingRecordAllocations::new(authority, Some(vec![expected]));
    let denial = pending
        .reserve(RecordAllocationClass::Entity, partition_id)
        .unwrap_err();
    assert_eq!(
        denial,
        RecordAllocationDenial::ReplayAppendFrontierMismatch {
            ordinal: 0,
            class: RecordAllocationClass::Entity,
            partition_id,
            expected_slot: 0,
            observed_slot: 1,
        }
    );
}

#[test]
fn metered_false_mask_and_heapsort_charge_every_chunk_comparison_and_swap() {
    let mut charges = Vec::new();
    let mut step = |units: u64| -> Result<(), &'static str> {
        charges.push(units);
        Ok(())
    };
    let mask = metered_false_mask(150, &mut step, |m| m, "mask").unwrap();
    assert_eq!(mask.len(), 150);
    assert!(mask.iter().all(|b| !b));
    assert_eq!(charges, vec![64, 64, 22]);

    // Exhausting work stops mask initialization before remaining chunks run.
    let mut budget = 64u64;
    let err = metered_false_mask(
        150,
        &mut |units| {
            if budget < units {
                return Err("budget-exhausted");
            }
            budget -= units;
            Ok(())
        },
        |m| m,
        "mask",
    )
    .unwrap_err();
    assert_eq!(err, "budget-exhausted");

    // Heapsort charges each comparison, sift swap, and extraction swap.
    // On [5, 1, 4, 2, 3]: 9 comparisons + 4 sift swaps + 4 extraction swaps = 17 exact units.
    let mut values = [5u32, 1, 4, 2, 3];
    let mut sort_units = 0u64;
    let mut sort_step = |units: u64| -> Result<(), &'static str> {
        sort_units += units;
        Ok(())
    };
    metered_heapsort(&mut values, |a, b| a.cmp(b), &mut sort_step).unwrap();
    assert_eq!(values, [1, 2, 3, 4, 5]);
    assert_eq!(
        sort_units, 17,
        "expected exact 9 comparisons + 4 sift swaps + 4 extraction swaps = 17 units"
    );

    // One-unit-short budget halts heapsort deterministically on the final operation.
    let mut short = [5u32, 1, 4, 2, 3];
    let mut remaining = sort_units - 1;
    let err = metered_heapsort(&mut short, |a, b| a.cmp(b), &mut |units| {
        if remaining < units {
            return Err("sort-stopped");
        }
        remaining -= units;
        Ok(())
    })
    .unwrap_err();
    assert_eq!(err, "sort-stopped");
    assert_eq!(remaining, 0);

    // Exact budget completes cleanly with zero remaining units.
    let mut exact = [5u32, 1, 4, 2, 3];
    let mut exact_remaining = sort_units;
    metered_heapsort(&mut exact, |a, b| a.cmp(b), &mut |units| {
        if exact_remaining < units {
            return Err("sort-stopped");
        }
        exact_remaining -= units;
        Ok(())
    })
    .unwrap();
    assert_eq!(exact, [1, 2, 3, 4, 5]);
    assert_eq!(exact_remaining, 0);
}
