//! Last-wins current inventory folding with bounded contiguous backing.

use super::*;

pub(super) fn fold_segments(
    source: &RecoverySelectedSourceInventory,
    updates: &[RecoverySegmentRoutingAction],
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<RecordSegmentPageManifestEntry>, CandidateBuildDenial> {
    let count = source
        .segment_pages
        .len()
        .checked_add(updates.len())
        .ok_or(CandidateBuildDenial::Invalid)?;
    let mut events =
        allowance.reserve::<(SegmentPageKey, usize, RecordSegmentPageManifestEntry)>(count)?;
    for (ordinal, page) in source.segment_pages.values().enumerate() {
        events.push((SegmentPageKey::from(page.entry), ordinal, page.entry));
    }
    for (index, update) in updates.iter().enumerate() {
        let ordinal = source
            .segment_pages
            .len()
            .checked_add(index)
            .ok_or(CandidateBuildDenial::Invalid)?;
        let entry = update.update();
        events.push((SegmentPageKey::from(entry), ordinal, entry));
    }
    events.sort_unstable_by_key(|event| (event.0, event.1));
    let mut result = allowance.reserve::<RecordSegmentPageManifestEntry>(events.len())?;
    let mut index = 0;
    while index < events.len() {
        let key = events[index].0;
        let mut next = index + 1;
        while next < events.len() && events[next].0 == key {
            next += 1;
        }
        result.push(events[next - 1].2);
        index = next;
    }
    release(events, allowance)?;
    Ok(result)
}

pub(super) fn fold_free(
    source: &RecoverySelectedSourceInventory,
    root_states: &[PersistedPhysicalRecoveryRootState],
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<RecordFreeSpaceManifestEntry>, CandidateBuildDenial> {
    let inline_count = root_states
        .iter()
        .try_fold(0_usize, |count, state| {
            count.checked_add(state.inline_allocations().len())
        })
        .ok_or(CandidateBuildDenial::Invalid)?;
    let count = source
        .free_entries
        .len()
        .checked_add(inline_count)
        .ok_or(CandidateBuildDenial::Invalid)?;
    let mut events =
        allowance.reserve::<(FreeSpaceKey, usize, Option<RecordFreeSpaceManifestEntry>)>(count)?;
    for (ordinal, entry) in source.free_entries.iter().copied().enumerate() {
        events.push((FreeSpaceKey::from(entry), ordinal, Some(entry)));
    }
    let mut ordinal = source.free_entries.len();
    for state in root_states {
        for allocation in state.inline_allocations() {
            let segment = allocation.segment();
            let key = FreeSpaceKey::inline(segment.segment_id().get())
                .ok_or(CandidateBuildDenial::Invalid)?;
            let entry = if allocation.used_pages() < allocation.page_capacity() {
                Some(
                    RecordFreeSpaceManifestEntry::inline_frontier(
                        segment.segment_id().get(),
                        u64::from(allocation.used_pages() + 1),
                        u64::from(allocation.page_capacity() - allocation.used_pages()),
                        segment.generation().get(),
                    )
                    .ok_or(CandidateBuildDenial::Invalid)?,
                )
            } else {
                None
            };
            events.push((key, ordinal, entry));
            ordinal = ordinal
                .checked_add(1)
                .ok_or(CandidateBuildDenial::Invalid)?;
        }
    }
    events.sort_unstable_by_key(|event| (event.0, event.1));
    let mut result = allowance.reserve::<RecordFreeSpaceManifestEntry>(events.len())?;
    let mut index = 0;
    while index < events.len() {
        let key = events[index].0;
        let mut next = index + 1;
        while next < events.len() && events[next].0 == key {
            next += 1;
        }
        if let Some(entry) = events[next - 1].2 {
            result.push(entry);
        }
        index = next;
    }
    release(events, allowance)?;
    Ok(result)
}

fn release<T>(
    values: Vec<T>,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(), CandidateBuildDenial> {
    let bytes = PlanningResidentAllowance::vector_bytes(&values)?;
    drop(values);
    allowance.release(bytes);
    Ok(())
}
