//! Sorted, bounded arena subtraction. Each projected range consumes one
//! predecessor free interval; adjacent intervals are never merged to admit it.

use super::{CandidateBuildDenial, RecoveryBaseImageAction, RecoverySelectedSourceInventory};
use crate::progression::planned::PlanningResidentAllowance;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, ExtentArenaId, ExtentArenaRange, FreeSpaceKey,
    RecordFreeSpaceManifestEntry,
};

pub(super) fn subtract(
    source: &RecoverySelectedSourceInventory,
    actions: &[RecoveryBaseImageAction],
    generation: u64,
    maximum_entries: u64,
    free: Vec<RecordFreeSpaceManifestEntry>,
    allowance: &mut PlanningResidentAllowance,
) -> Result<(Vec<RecordFreeSpaceManifestEntry>, u64), CandidateBuildDenial> {
    let range_count = actions
        .iter()
        .filter(|action| {
            action.is_projected()
                && matches!(
                    action.placement(),
                    CurrentPhysicalRecordPlacement::Extent(_)
                )
        })
        .count();
    let mut ranges = allowance.reserve::<ExtentArenaRange>(range_count)?;
    for action in actions.iter().filter(|action| action.is_projected()) {
        if let CurrentPhysicalRecordPlacement::Extent(extent) = action.placement() {
            ranges.push(extent.arena_range());
        }
    }
    ranges.sort_unstable_by_key(|range| (range.arena(), range.offset()));

    let first = source.free_space.next_arena();
    let next = ranges.iter().try_fold(first, |next, range| {
        range
            .arena()
            .get()
            .checked_add(1)
            .map(|after| next.max(after))
            .ok_or(CandidateBuildDenial::Invalid)
    })?;
    let new_count = next
        .checked_sub(first)
        .filter(|count| *count <= maximum_entries)
        .and_then(|count| usize::try_from(count).ok())
        .ok_or(CandidateBuildDenial::Invalid)?;
    let event_count = free
        .len()
        .checked_add(new_count)
        .ok_or(CandidateBuildDenial::Invalid)?;
    let mut events =
        allowance.reserve::<(FreeSpaceKey, usize, RecordFreeSpaceManifestEntry)>(event_count)?;
    for (ordinal, entry) in free.iter().copied().enumerate() {
        events.push((FreeSpaceKey::from(entry), ordinal, entry));
    }
    for (offset, arena_id) in (first..next).enumerate() {
        let arena = ExtentArenaId::new(arena_id).ok_or(CandidateBuildDenial::Invalid)?;
        let range = ExtentArenaRange::new(arena, 0, source.free_space.arena_capacity())
            .ok_or(CandidateBuildDenial::Invalid)?;
        let entry = RecordFreeSpaceManifestEntry::arena_range(range, generation)
            .ok_or(CandidateBuildDenial::Invalid)?;
        let ordinal = free
            .len()
            .checked_add(offset)
            .ok_or(CandidateBuildDenial::Invalid)?;
        events.push((FreeSpaceKey::from(entry), ordinal, entry));
    }
    events.sort_unstable_by_key(|event| (event.0, event.1));

    // Each consumed range can add at most one interval. Pushes cannot grow
    // the charged result backing, even after repeated splits of one interval.
    let output_count = event_count
        .checked_add(ranges.len())
        .ok_or(CandidateBuildDenial::Invalid)?;
    let mut result = allowance.reserve::<RecordFreeSpaceManifestEntry>(output_count)?;
    let mut range_index = 0;
    let mut event_index = 0;
    while event_index < events.len() {
        let key = events[event_index].0;
        let mut end = event_index + 1;
        while end < events.len() && events[end].0 == key {
            end += 1;
        }
        let entry = events[end - 1].2;
        match entry.arena_free_range() {
            None => result.push(entry),
            Some(available) => {
                let mut cursor = available.offset();
                let mut consumed = false;
                while let Some(range) = ranges.get(range_index).copied() {
                    if range.arena() > available.arena()
                        || (range.arena() == available.arena() && range.offset() >= available.end())
                    {
                        break;
                    }
                    if range.arena() != available.arena()
                        || range.offset() < cursor
                        || range.end() > available.end()
                        || range.offset() % source.free_space.arena_alignment() != 0
                        || range.length() % source.free_space.arena_alignment() != 0
                        || range.end() > source.free_space.arena_capacity()
                    {
                        return Err(CandidateBuildDenial::Invalid);
                    }
                    if cursor < range.offset() {
                        result.push(residual(
                            available.arena(),
                            cursor,
                            range.offset(),
                            generation,
                        )?);
                    }
                    cursor = range.end();
                    consumed = true;
                    range_index += 1;
                }
                if consumed {
                    if cursor < available.end() {
                        result.push(residual(
                            available.arena(),
                            cursor,
                            available.end(),
                            generation,
                        )?);
                    }
                } else {
                    result.push(entry);
                }
            }
        }
        event_index = end;
    }
    if range_index != ranges.len() {
        return Err(CandidateBuildDenial::Invalid);
    }
    release(free, allowance)?;
    release(events, allowance)?;
    release(ranges, allowance)?;
    Ok((result, next))
}

fn residual(
    arena: ExtentArenaId,
    first: u64,
    end: u64,
    generation: u64,
) -> Result<RecordFreeSpaceManifestEntry, CandidateBuildDenial> {
    let range =
        ExtentArenaRange::new(arena, first, end - first).ok_or(CandidateBuildDenial::Invalid)?;
    RecordFreeSpaceManifestEntry::arena_range(range, generation)
        .ok_or(CandidateBuildDenial::Invalid)
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
