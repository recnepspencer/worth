use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, FreeSpaceKey, PersistedPhysicalRecoveryRootState,
    RecordFreeSpaceManifestEntry, RecordSegmentPageManifestEntry, SegmentPageKey,
};

use super::CandidateBuildDenial;
mod arenas;
mod fold;
#[cfg(test)]
mod tests;
use crate::progression::planned::PlanningResidentAllowance;
use crate::progression::{
    RecoveryBaseImageAction, RecoverySegmentRoutingAction, RecoverySelectedSourceInventory,
};
use fold::{fold_free, fold_segments};

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

impl FinalInventory {
    pub(super) fn release(
        self,
        allowance: &mut PlanningResidentAllowance,
    ) -> Result<(), CandidateBuildDenial> {
        let bytes = PlanningResidentAllowance::vector_bytes(&self.placements)?
            .checked_add(PlanningResidentAllowance::vector_bytes(&self.segments)?)
            .and_then(|bytes| {
                bytes.checked_add(PlanningResidentAllowance::vector_bytes(&self.free).ok()?)
            })
            .ok_or(CandidateBuildDenial::Invalid)?;
        drop(self);
        allowance.release(bytes);
        Ok(())
    }
}

pub(super) fn finalize(
    source: &RecoverySelectedSourceInventory,
    actions: &[RecoveryBaseImageAction],
    updates: &[RecoverySegmentRoutingAction],
    root_states: &[PersistedPhysicalRecoveryRootState],
    selected_capacity: u16,
    generation: u64,
    maximum_entries: u64,
    allowance: &mut PlanningResidentAllowance,
) -> Result<FinalInventory, CandidateBuildDenial> {
    let capacity = common_capacity(root_states, selected_capacity)?;
    let mut placements = allowance.reserve::<CurrentPhysicalRecordPlacement>(actions.len())?;
    placements.extend(actions.iter().map(|action| action.placement()));
    placements.sort_unstable_by_key(|placement| placement.record());
    if placements
        .windows(2)
        .any(|pair| pair[0].record() == pair[1].record())
    {
        return Err(CandidateBuildDenial::Invalid);
    }
    let segments = fold_segments(source, updates, allowance)?;
    let free = fold_free(source, root_states, allowance)?;
    let next_segment = next_segment(source, root_states)?;
    let next_page = next_page(source, &placements)?;
    let next_extent = next_extent(source, &placements)?;
    let (free, next_arena) = arenas::subtract(
        source,
        actions,
        generation,
        maximum_entries,
        free,
        allowance,
    )?;
    let (last_inline_record, last_inline_segment) = root_states
        .iter()
        .rev()
        .find_map(|state| state.last_inline_record().zip(state.last_inline_segment()))
        .map_or((None, None), |(record, segment)| {
            (Some(record), Some(segment))
        });
    Ok(FinalInventory {
        placements,
        segments,
        free,
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
