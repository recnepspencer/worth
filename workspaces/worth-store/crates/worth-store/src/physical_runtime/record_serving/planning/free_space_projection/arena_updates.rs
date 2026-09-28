use super::{damaged, FreeSpaceUpdate};
use crate::physical_runtime::record_serving::{
    access::manifest_routing::ManifestDiscoveryCounterSnapshot, arena::ExtentArenaCapacity,
    planning::free_space_routing::FreeSpaceReader, RecordAppendError,
};
use std::collections::BTreeMap;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableFreeSpaceManifestHeader, ExtentArenaId,
    ExtentArenaRange, FreeSpaceKey, RecordFreeSpaceManifestEntry,
};

/// Project only settled routes. Other in-flight claims remain published free
/// and are excluded solely by the runtime reservation owner until they settle.
pub(super) fn subtract_allocations(
    reader: &FreeSpaceReader<'_>,
    allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    current: &DurableFreeSpaceManifestHeader,
    capacity: ExtentArenaCapacity,
    generation: u64,
    next_arena: u64,
    placements: impl Iterator<Item = CurrentPhysicalRecordPlacement>,
    updates: &mut BTreeMap<FreeSpaceKey, FreeSpaceUpdate>,
    counters: &mut ManifestDiscoveryCounterSnapshot,
) -> Result<(), RecordAppendError> {
    for id in current.next_arena()..next_arena {
        let arena = ExtentArenaId::new(id).ok_or_else(damaged)?;
        insert_available(
            updates,
            ExtentArenaRange::new(arena, 0, capacity.get()).ok_or_else(damaged)?,
            generation,
        )?;
    }
    let mut ranges = placements
        .filter_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Extent(extent) => Some(extent.arena_range()),
            _ => None,
        })
        .collect::<Vec<_>>();
    ranges.sort_unstable_by_key(|range| (range.arena(), range.offset()));
    for range in ranges {
        let key = FreeSpaceKey::arena(range.arena(), range.offset());
        let staged = updates
            .range(..=key)
            .next_back()
            .and_then(|(_, update)| match update {
                FreeSpaceUpdate::Available(entry) => entry
                    .arena_free_range()
                    .filter(|free| contains(*free, range)),
                FreeSpaceUpdate::Exhausted => None,
            });
        let free = match staged {
            Some(free) => free,
            None => {
                let entry = reader
                    .floor(allocation, key, counters)
                    .map_err(|_| damaged())?
                    .ok_or_else(damaged)?;
                if updates.contains_key(&FreeSpaceKey::from(entry)) {
                    return Err(damaged());
                }
                entry
                    .arena_free_range()
                    .filter(|free| contains(*free, range))
                    .ok_or_else(damaged)?
            }
        };
        updates.insert(
            FreeSpaceKey::arena(free.arena(), free.offset()),
            FreeSpaceUpdate::Exhausted,
        );
        if free.offset() < range.offset() {
            insert_available(
                updates,
                ExtentArenaRange::new(free.arena(), free.offset(), range.offset() - free.offset())
                    .ok_or_else(damaged)?,
                generation,
            )?;
        }
        if range.end() < free.end() {
            insert_available(
                updates,
                ExtentArenaRange::new(free.arena(), range.end(), free.end() - range.end())
                    .ok_or_else(damaged)?,
                generation,
            )?;
        }
    }
    Ok(())
}

fn contains(free: ExtentArenaRange, range: ExtentArenaRange) -> bool {
    free.arena() == range.arena() && free.offset() <= range.offset() && free.end() >= range.end()
}

fn insert_available(
    updates: &mut BTreeMap<FreeSpaceKey, FreeSpaceUpdate>,
    range: ExtentArenaRange,
    generation: u64,
) -> Result<(), RecordAppendError> {
    updates.insert(
        FreeSpaceKey::arena(range.arena(), range.offset()),
        FreeSpaceUpdate::Available(
            RecordFreeSpaceManifestEntry::arena_range(range, generation).ok_or_else(damaged)?,
        ),
    );
    Ok(())
}
