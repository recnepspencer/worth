use super::*;
use std::collections::BTreeMap;
use worth_store_physical_format::{
    DurableExtentRecordPlacement, DurableFreeSpaceManifestHeader, ExtentArenaId, ExtentArenaRange,
    FreeSpaceBlockReference, PersistedInlineSegmentAllocation, PersistedRecordIdentity,
    PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalSegmentId,
};

fn allowance() -> PlanningResidentAllowance {
    PlanningResidentAllowance::new(0, 1 << 20).unwrap()
}

fn source(
    entries: Vec<RecordFreeSpaceManifestEntry>,
    next_arena: u64,
) -> RecoverySelectedSourceInventory {
    let root = entries.first().zip(entries.last()).map(|(first, last)| {
        FreeSpaceBlockReference::new(
            1,
            1,
            0,
            1,
            FreeSpaceKey::from(*first),
            FreeSpaceKey::from(*last),
        )
        .unwrap()
    });
    let free_space = DurableFreeSpaceManifestHeader::new(
        1,
        7,
        4,
        4,
        entries.len() as u64,
        1,
        1,
        1,
        next_arena,
        32768,
        4096,
        if root.is_some() { 2 } else { 1 },
        root,
    )
    .unwrap();
    RecoverySelectedSourceInventory {
        free_space,
        segment_pages: BTreeMap::new(),
        segment_topology: BTreeMap::new(),
        free_entries: entries.into_boxed_slice(),
        free_topology: BTreeMap::new(),
        source_artifacts: Box::new([]),
    }
}

fn segment_entry(block: u64) -> RecordSegmentPageManifestEntry {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = PhysicalSegmentId::from_raw(1).unwrap();
    let generation = PhysicalGeneration::from_raw(1).unwrap();
    RecordSegmentPageManifestEntry::new(
        authority
            .page_cell(segment, PhysicalPageId::from_raw(1).unwrap())
            .with_page_generation(generation),
        authority
            .segment_cell(segment)
            .with_segment_generation(generation),
        u32::try_from(block).unwrap(),
        0,
    )
    .unwrap()
}

#[test]
fn repeated_segment_update_keeps_last_ordinal_and_exhausted_inline_is_removed() {
    let inline = RecordFreeSpaceManifestEntry::inline_frontier(1, 1, 4, 1).unwrap();
    let source = source(vec![inline], 1);
    let updates = [
        RecoverySegmentRoutingAction {
            ordinal: 1,
            update: segment_entry(2),
        },
        RecoverySegmentRoutingAction {
            ordinal: 2,
            update: segment_entry(3),
        },
    ];
    let segments = fold_segments(&source, &updates, &mut allowance()).unwrap();
    assert_eq!(segments, vec![segment_entry(3)]);

    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let segment = authority
        .segment_cell(PhysicalSegmentId::from_raw(1).unwrap())
        .with_segment_generation(PhysicalGeneration::from_raw(2).unwrap());
    let exhausted = PersistedInlineSegmentAllocation::new(segment, 4, 4).unwrap();
    let state =
        PersistedPhysicalRecoveryRootState::new(1, 1, 4, vec![exhausted], None, None).unwrap();
    assert!(fold_free(&source, &[state], &mut allowance())
        .unwrap()
        .is_empty());
}

fn arena_entry(offset: u64, length: u64, generation: u64) -> RecordFreeSpaceManifestEntry {
    RecordFreeSpaceManifestEntry::arena_range(
        ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), offset, length).unwrap(),
        generation,
    )
    .unwrap()
}

fn projected(ordinal: u64, offset: u64) -> RecoveryBaseImageAction {
    let authority = PhysicalGenerationAuthority::for_canonical_physical_format();
    let extent = authority
        .record_extent_cell(PhysicalExtentId::from_raw(ordinal).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    let record = PersistedRecordIdentity::new([9; 16], ordinal).unwrap();
    let range = ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), offset, 4096).unwrap();
    let placement = CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::legacy_unknown(record, extent, 23, range).unwrap(),
    );
    RecoveryBaseImageAction::ProjectRecoveryPlacement { ordinal, placement }
}

#[test]
fn two_ranges_split_one_arena_without_changing_untouched_generation() {
    let old = vec![arena_entry(0, 16384, 1), arena_entry(20480, 4096, 1)];
    let source = source(old.clone(), 2);
    let actions = [projected(1, 4096), projected(2, 12288)];
    let (result, next_arena) =
        arenas::subtract(&source, &actions, 2, 8, old, &mut allowance()).unwrap();
    assert_eq!(next_arena, 2);
    assert_eq!(
        result,
        vec![
            arena_entry(0, 4096, 2),
            arena_entry(8192, 4096, 2),
            arena_entry(20480, 4096, 1),
        ]
    );
}
