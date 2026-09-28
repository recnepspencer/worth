use std::collections::BTreeMap;

use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, FreeSpaceKey, PersistedPhysicalRecoveryRootState,
    RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry, SegmentPageKey,
};

use super::CandidateBuildDenial;
mod arenas;
use crate::progression::{
    RecoveryBaseImageAction, RecoverySegmentRoutingAction, RecoverySelectedSourceInventory,
};

pub(super) struct FinalInventory {
    pub(super) placements: Vec<CurrentPhysicalRecordPlacement>,
    pub(super) segments: Vec<RecordSegmentPageManifestEntry>,
    pub(super) free: Vec<RecordFreeSpaceManifestEntry>,
    pub(super) next_segment: u64,
    pub(super) next_page: u64,
    pub(super) next_extent: u64,
    pub(super) next_arena: u64,
    pub(super) capacity: u16,
    pub(super) last_inline_record: Option<worth_store_physical_format::PersistedRecordIdentity>,
    pub(super) last_inline_segment: Option<worth_store_physical_format::SegmentGenerationCell>,
}

pub(super) fn finalize(
    source: &RecoverySelectedSourceInventory,
    actions: &[RecoveryBaseImageAction],
    updates: &[RecoverySegmentRoutingAction],
    root_states: &[PersistedPhysicalRecoveryRootState],
    selected_capacity: u16,
    generation: u64,
    maximum_entries: u64,
) -> Result<FinalInventory, CandidateBuildDenial> {
    let capacity = common_capacity(root_states, selected_capacity)?;
    let mut placements = actions
        .iter()
        .map(|action| action.placement())
        .collect::<Vec<_>>();
    placements.sort_unstable_by_key(|placement| placement.record());
    if placements
        .windows(2)
        .any(|pair| pair[0].record() == pair[1].record())
    {
        return Err(CandidateBuildDenial::Invalid);
    }
    let mut segments = source
        .segment_pages
        .values()
        .map(|page| (SegmentPageKey::from(page.entry), page.entry))
        .collect::<BTreeMap<_, _>>();
    for update in updates {
        let entry = update.update();
        segments.insert(SegmentPageKey::from(entry), entry);
    }
    let mut free = source
        .free_entries
        .iter()
        .copied()
        .map(|entry| (FreeSpaceKey::from(entry), entry))
        .collect::<BTreeMap<_, _>>();
    for state in root_states {
        for allocation in state.inline_allocations() {
            let segment = allocation.segment();
            let key = FreeSpaceKey::inline(segment.segment_id().get())
                .ok_or(CandidateBuildDenial::Invalid)?;
            if allocation.used_pages() < allocation.page_capacity() {
                let entry = RecordFreeSpaceManifestEntry::inline_frontier(
                    segment.segment_id().get(),
                    u64::from(allocation.used_pages() + 1),
                    u64::from(allocation.page_capacity() - allocation.used_pages()),
                    segment.generation().get(),
                )
                .ok_or(CandidateBuildDenial::Invalid)?;
                free.insert(key, entry);
            } else {
                free.remove(&key);
            }
        }
    }
    let next_segment = next_segment(source, root_states)?;
    let next_page = next_page(source, &placements)?;
    let next_extent = next_extent(source, &placements)?;
    let next_arena = arenas::subtract(source, actions, generation, maximum_entries, &mut free)?;
    let (last_inline_record, last_inline_segment) = root_states
        .iter()
        .rev()
        .find_map(|state| state.last_inline_record().zip(state.last_inline_segment()))
        .map_or((None, None), |(record, segment)| {
            (Some(record), Some(segment))
        });
    Ok(FinalInventory {
        placements,
        segments: segments.into_values().collect(),
        free: free.into_values().collect(),
        next_segment,
        next_page,
        next_extent,
        next_arena,
        capacity,
        last_inline_record,
        last_inline_segment,
    })
}

fn common_capacity(
    states: &[PersistedPhysicalRecoveryRootState],
    selected_capacity: u16,
) -> Result<u16, CandidateBuildDenial> {
    let first = states.first().map_or(selected_capacity, |state| {
        state.successor_manifest_capacity()
    });
    states
        .iter()
        .all(|state| state.successor_manifest_capacity() == first)
        .then_some(first)
        .ok_or(CandidateBuildDenial::Invalid)
}

fn next_segment(
    source: &RecoverySelectedSourceInventory,
    states: &[PersistedPhysicalRecoveryRootState],
) -> Result<u64, CandidateBuildDenial> {
    states
        .iter()
        .flat_map(|state| state.inline_allocations())
        .try_fold(source.free_space.next_segment(), |next, allocation| {
            allocation
                .segment()
                .segment_id()
                .get()
                .checked_add(1)
                .map(|candidate| next.max(candidate))
                .ok_or(CandidateBuildDenial::Invalid)
        })
}

fn next_page(
    source: &RecoverySelectedSourceInventory,
    placements: &[CurrentPhysicalRecordPlacement],
) -> Result<u64, CandidateBuildDenial> {
    placements
        .iter()
        .filter_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Inline(inline) => Some(inline.page().get()),
            CurrentPhysicalRecordPlacement::Extent(_) => None,
        })
        .try_fold(source.free_space.next_page(), |next, page| {
            page.checked_add(1)
                .map(|candidate| next.max(candidate))
                .ok_or(CandidateBuildDenial::Invalid)
        })
}

fn next_extent(
    source: &RecoverySelectedSourceInventory,
    placements: &[CurrentPhysicalRecordPlacement],
) -> Result<u64, CandidateBuildDenial> {
    placements
        .iter()
        .filter_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Extent(extent) => Some(extent.extent().get()),
            CurrentPhysicalRecordPlacement::Inline(_) => None,
        })
        .try_fold(source.free_space.next_extent(), |next, extent| {
            extent
                .checked_add(1)
                .map(|candidate| next.max(candidate))
                .ok_or(CandidateBuildDenial::Invalid)
        })
}
