use super::{CandidateBuildDenial, RecoveryBaseImageAction, RecoverySelectedSourceInventory};
use std::collections::BTreeMap;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, ExtentArenaId, ExtentArenaRange, FreeSpaceKey,
    RecordFreeSpaceManifestEntry,
};

pub(super) fn subtract(
    source: &RecoverySelectedSourceInventory,
    actions: &[RecoveryBaseImageAction],
    generation: u64,
    maximum_entries: u64,
    free: &mut BTreeMap<FreeSpaceKey, RecordFreeSpaceManifestEntry>,
) -> Result<u64, CandidateBuildDenial> {
    let mut ranges = actions
        .iter()
        .filter(|action| action.is_projected())
        .filter_map(|action| match action.placement() {
            CurrentPhysicalRecordPlacement::Extent(extent) => Some(extent.arena_range()),
            _ => None,
        })
        .collect::<Vec<_>>();
    ranges.sort_unstable_by_key(|range| (range.arena(), range.offset()));
    let first = source.free_space.next_arena();
    let next = ranges.iter().try_fold(first, |next, range| {
        range
            .arena()
            .get()
            .checked_add(1)
            .map(|candidate| next.max(candidate))
            .ok_or(CandidateBuildDenial::Invalid)
    })?;
    // Untrusted arena identifiers cannot cause unbounded gap enumeration.
    if next
        .checked_sub(first)
        .is_none_or(|count| count > maximum_entries)
    {
        return Err(CandidateBuildDenial::Invalid);
    }
    for arena in first..next {
        let arena = ExtentArenaId::new(arena).ok_or(CandidateBuildDenial::Invalid)?;
        insert(
            free,
            ExtentArenaRange::new(arena, 0, source.free_space.arena_capacity())
                .ok_or(CandidateBuildDenial::Invalid)?,
            generation,
        )?;
    }
    for range in ranges {
        if range.end() > source.free_space.arena_capacity()
            || range.offset() % source.free_space.arena_alignment() != 0
            || range.length() % source.free_space.arena_alignment() != 0
        {
            return Err(CandidateBuildDenial::Invalid);
        }
        let key = FreeSpaceKey::arena(range.arena(), range.offset());
        let available = free
            .range(..=key)
            .next_back()
            .and_then(|(_, entry)| entry.arena_free_range())
            .filter(|available| {
                available.arena() == range.arena()
                    && available.offset() <= range.offset()
                    && available.end() >= range.end()
            })
            .ok_or(CandidateBuildDenial::Invalid)?;
        free.remove(&FreeSpaceKey::arena(available.arena(), available.offset()));
        if available.offset() < range.offset() {
            insert(
                free,
                ExtentArenaRange::new(
                    range.arena(),
                    available.offset(),
                    range.offset() - available.offset(),
                )
                .ok_or(CandidateBuildDenial::Invalid)?,
                generation,
            )?;
        }
        if range.end() < available.end() {
            insert(
                free,
                ExtentArenaRange::new(range.arena(), range.end(), available.end() - range.end())
                    .ok_or(CandidateBuildDenial::Invalid)?,
                generation,
            )?;
        }
    }
    Ok(next)
}

fn insert(
    free: &mut BTreeMap<FreeSpaceKey, RecordFreeSpaceManifestEntry>,
    range: ExtentArenaRange,
    generation: u64,
) -> Result<(), CandidateBuildDenial> {
    free.insert(
        FreeSpaceKey::arena(range.arena(), range.offset()),
        RecordFreeSpaceManifestEntry::arena_range(range, generation)
            .ok_or(CandidateBuildDenial::Invalid)?,
    );
    Ok(())
}
