use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, DurableInlineRecordPlacement,
    PersistedRecordIdentity, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalRecordSlot,
};

use super::*;
use crate::physical_runtime::record_serving::planning::{
    allocation_frontier::RecordAllocationFrontier, inline_segment_plan::WorkingSegment,
};

#[test]
fn unpublished_reservations_do_not_enter_any_committed_identity_frontier() {
    let current = DurableFreeSpaceManifestHeader::new(1, 1, 16, 4, 0, 1, 1, 1, 1, None).unwrap();
    let mut live = RecordAllocationFrontier::new(&current);
    let mut publishing = live.reserve(64, 64, 8).unwrap();
    let mut abandoned = live.reserve(64, 64, 8).unwrap();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let generation = PhysicalGeneration::from_raw(1).unwrap();
    let mut allocations = Vec::new();
    let mut placements = Vec::new();
    for ordinal in 1..=4 {
        let segment = authority
            .segment_cell(publishing.allocate_segment().unwrap())
            .with_segment_generation(generation);
        let page = authority
            .page_cell(segment.segment_id(), publishing.allocate_page().unwrap())
            .with_page_generation(generation);
        let record = PersistedRecordIdentity::new([0x31; 16], ordinal).unwrap();
        let slot = authority
            .slot_cell(
                segment.segment_id(),
                page.page_id(),
                PhysicalRecordSlot::from_raw(1).unwrap(),
            )
            .with_slot_generation(generation);
        placements.push(CurrentPhysicalRecordPlacement::Inline(
            DurableInlineRecordPlacement::new(record, segment, page, slot, 4, 16).unwrap(),
        ));
        allocations.push(
            WorkingSegment {
                segment,
                page_capacity: 4,
                used_pages: 1,
                membership_updates: Vec::new(),
                data_pages: Vec::new(),
            }
            .allocation(),
        );
    }
    let extent = authority
        .record_extent_cell(publishing.allocate_extent().unwrap())
        .with_extent_generation(generation);
    placements.push(CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::new(
            PersistedRecordIdentity::new([0x31; 16], 5).unwrap(),
            extent,
            65536,
        )
        .unwrap(),
    ));
    let committed = CommittedAllocationFrontier::from_publication(
        &current,
        &allocations,
        placements.into_iter(),
    )
    .unwrap();
    assert_eq!(
        (
            committed.next_segment,
            committed.next_page,
            committed.next_extent
        ),
        (5, 5, 2)
    );
    assert_eq!(
        (live.next_segment(), live.next_page(), live.next_extent()),
        (129, 129, 17)
    );
    assert_eq!(abandoned.allocate_segment().unwrap().get(), 65);
    assert_eq!(abandoned.allocate_page().unwrap().get(), 65);
    assert_eq!(abandoned.allocate_extent().unwrap().get(), 9);
    let mut after = live.reserve(1, 1, 1).unwrap();
    assert_eq!(after.allocate_segment().unwrap().get(), 129);
    assert_eq!(after.allocate_page().unwrap().get(), 129);
    assert_eq!(after.allocate_extent().unwrap().get(), 17);
}

#[test]
fn no_materialization_preserves_the_previous_durable_frontier() {
    let current =
        DurableFreeSpaceManifestHeader::new(1, 1, 16, 4, 0, 200, 300, 400, 1, None).unwrap();
    let committed =
        CommittedAllocationFrontier::from_publication(&current, &[], std::iter::empty()).unwrap();
    assert_eq!(
        (
            committed.next_segment,
            committed.next_page,
            committed.next_extent
        ),
        (200, 300, 400)
    );
    let mut next = 300;
    assert_eq!(advance(&mut next, 99), Some(()));
    assert_eq!(next, 300);
    assert_eq!(advance(&mut next, u64::MAX), None);
    assert_eq!(next, 300);
}

#[test]
fn later_concrete_publication_preserves_gaps_left_by_abandoned_reservations() {
    let current = DurableFreeSpaceManifestHeader::new(1, 1, 16, 4, 0, 1, 1, 1, 1, None).unwrap();
    let mut live = RecordAllocationFrontier::new(&current);
    let _abandoned = live.reserve(64, 64, 8).unwrap();
    let mut later = live.reserve(4, 4, 2).unwrap();
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let generation = PhysicalGeneration::from_raw(1).unwrap();
    let segment = authority
        .segment_cell(later.allocate_segment().unwrap())
        .with_segment_generation(generation);
    let page = authority
        .page_cell(segment.segment_id(), later.allocate_page().unwrap())
        .with_page_generation(generation);
    let slot = authority
        .slot_cell(
            segment.segment_id(),
            page.page_id(),
            PhysicalRecordSlot::from_raw(1).unwrap(),
        )
        .with_slot_generation(generation);
    let inline = DurableInlineRecordPlacement::new(
        PersistedRecordIdentity::new([0x31; 16], 1).unwrap(),
        segment,
        page,
        slot,
        4,
        16,
    )
    .unwrap();
    let extent = DurableExtentRecordPlacement::new(
        PersistedRecordIdentity::new([0x31; 16], 2).unwrap(),
        authority
            .record_extent_cell(later.allocate_extent().unwrap())
            .with_extent_generation(generation),
        65536,
    )
    .unwrap();
    let allocation = WorkingSegment {
        segment,
        page_capacity: 4,
        used_pages: 1,
        membership_updates: Vec::new(),
        data_pages: Vec::new(),
    }
    .allocation();
    let committed = CommittedAllocationFrontier::from_publication(
        &current,
        &[allocation],
        [
            CurrentPhysicalRecordPlacement::Inline(inline),
            CurrentPhysicalRecordPlacement::Extent(extent),
        ]
        .into_iter(),
    )
    .unwrap();
    assert_eq!(
        (
            committed.next_segment,
            committed.next_page,
            committed.next_extent
        ),
        (66, 66, 10)
    );
    assert_eq!(
        (live.next_segment(), live.next_page(), live.next_extent()),
        (69, 69, 11)
    );
}
