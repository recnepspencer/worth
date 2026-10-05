use super::*;
pub(super) fn next_extent(
    source: &DurableFreeSpaceManifestHeader,
    routes: &[CurrentPhysicalRecordPlacement],
) -> Option<u64> {
    routes
        .iter()
        .filter_map(|route| match route {
            CurrentPhysicalRecordPlacement::Extent(extent) => Some(extent.extent().get()),
            CurrentPhysicalRecordPlacement::Inline(_) => None,
        })
        .try_fold(source.next_extent(), |next, extent| {
            extent.checked_add(1).map(|candidate| next.max(candidate))
        })
}

pub(super) fn routes_match(
    source: &[CurrentPhysicalRecordPlacement],
    result: &[CurrentPhysicalRecordPlacement],
    dropped: &[PersistedRecordIdentity],
    projected: &[CurrentPhysicalRecordPlacement],
) -> bool {
    let mut old = source
        .iter()
        .copied()
        .filter(|route| dropped.binary_search(&route.record()).is_err())
        .peekable();
    let mut new = projected.iter().copied().peekable();
    for actual in result {
        let expected = match (old.peek(), new.peek()) {
            (Some(a), Some(b)) if a.record() < b.record() => old.next(),
            (Some(a), Some(b)) if a.record() == b.record() => return false,
            (_, Some(_)) => new.next(),
            (Some(_), None) => old.next(),
            (None, None) => return false,
        };
        if expected != Some(*actual) {
            return false;
        }
    }
    old.next().is_none() && new.next().is_none()
}

pub(super) fn validate_arenas(
    source: ReleasedInventoryView<'_>,
    result: ReleasedInventoryView<'_>,
    projected: &[CurrentPhysicalRecordPlacement],
    maximum_entries: u64,
    maximum_scratch: u64,
) -> Result<u64, ReleasedV3InventoryTransitionDenial> {
    use ReleasedV3InventoryTransitionDenial as Denial;
    let mut ranges = reserve::<ExtentArenaRange>(projected.len(), 0, maximum_scratch)?;
    ranges.extend(projected.iter().filter_map(|route| match route {
        CurrentPhysicalRecordPlacement::Extent(extent) => Some(extent.arena_range()),
        CurrentPhysicalRecordPlacement::Inline(_) => None,
    }));
    ranges.sort_unstable_by_key(|range| (range.arena(), range.offset()));
    let first = source.free.next_arena();
    let next = ranges
        .iter()
        .try_fold(first, |next, range| {
            range
                .arena()
                .get()
                .checked_add(1)
                .map(|candidate| next.max(candidate))
        })
        .ok_or(Denial::InvalidDelta)?;
    if next - first > maximum_entries {
        let past = super::ExceededRootHistoryBound::entries(next - first, maximum_entries);
        return Err(Denial::BoundExceeded(past));
    }
    for range in &ranges {
        if range.end() > source.free.arena_capacity()
            || range.offset() % source.free.arena_alignment() != 0
            || range.length() % source.free.arena_alignment() != 0
        {
            return Err(Denial::InvalidDelta);
        }
    }
    let generation = result.root.generation();
    let mut source_free = source.free_entries.iter().copied().peekable();
    let mut new_arena = first;
    let mut result_free = result.free_entries.iter().copied();
    let mut projected_ranges = ranges.into_iter().peekable();
    while source_free.peek().is_some() || new_arena < next {
        let new_key = if new_arena < next {
            Some(FreeSpaceKey::arena(
                ExtentArenaId::new(new_arena).ok_or(Denial::InvalidDelta)?,
                0,
            ))
        } else {
            None
        };
        let entry = match (source_free.peek(), new_key) {
            (Some(old), Some(new)) if FreeSpaceKey::from(*old) < new => source_free.next().unwrap(),
            (Some(old), Some(new)) if FreeSpaceKey::from(*old) == new => {
                return Err(Denial::InvalidDelta)
            }
            (_, Some(_)) => {
                let range = ExtentArenaRange::new(
                    ExtentArenaId::new(new_arena).ok_or(Denial::InvalidDelta)?,
                    0,
                    source.free.arena_capacity(),
                )
                .ok_or(Denial::InvalidDelta)?;
                new_arena += 1;
                RecordFreeSpaceManifestEntry::arena_range(range, generation)
                    .ok_or(Denial::InvalidDelta)?
            }
            (Some(_), None) => source_free.next().unwrap(),
            (None, None) => unreachable!(),
        };
        let Some(available) = entry.arena_free_range() else {
            if result_free.next() != Some(entry) {
                return Err(Denial::InvalidDelta);
            }
            continue;
        };
        let mut cursor = available.offset();
        let mut touched = false;
        while let Some(range) = projected_ranges.peek().copied() {
            if (range.arena(), range.offset()) >= (available.arena(), available.end()) {
                break;
            }
            if range.arena() != available.arena()
                || range.offset() < cursor
                || range.end() > available.end()
            {
                return Err(Denial::InvalidDelta);
            }
            if cursor < range.offset() {
                emit_fragment(
                    &mut result_free,
                    available.arena(),
                    cursor,
                    range.offset(),
                    generation,
                )?;
            }
            cursor = range.end();
            touched = true;
            projected_ranges.next();
        }
        if !touched {
            if result_free.next() != Some(entry) {
                return Err(Denial::InvalidDelta);
            }
        } else if cursor < available.end() {
            emit_fragment(
                &mut result_free,
                available.arena(),
                cursor,
                available.end(),
                generation,
            )?;
        }
    }
    if projected_ranges.next().is_some() || result_free.next().is_some() {
        return Err(Denial::InvalidDelta);
    }
    Ok(next)
}

fn emit_fragment(
    result: &mut impl Iterator<Item = RecordFreeSpaceManifestEntry>,
    arena: ExtentArenaId,
    start: u64,
    end: u64,
    generation: u64,
) -> Result<(), ReleasedV3InventoryTransitionDenial> {
    use ReleasedV3InventoryTransitionDenial as Denial;
    let range = ExtentArenaRange::new(arena, start, end - start).ok_or(Denial::InvalidDelta)?;
    let expected =
        RecordFreeSpaceManifestEntry::arena_range(range, generation).ok_or(Denial::InvalidDelta)?;
    if result.next() != Some(expected) {
        return Err(Denial::InvalidDelta);
    }
    Ok(())
}

pub(super) fn root_semantics_match(
    source: ReleasedInventoryView<'_>,
    result: ReleasedInventoryView<'_>,
    dropped: &[PersistedRecordIdentity],
    projected: &[CurrentPhysicalRecordPlacement],
    head_replay: Option<&VerifiedSelectedReleaseHeadReplayV14>,
    directory_replacement: Option<&VerifiedReleasedDirectoryReplacement>,
) -> bool {
    let removed = |record| dropped.binary_search(&record).is_ok();
    let invalidates_directory_watermark =
        source
            .root
            .derived_family_directory()
            .is_some_and(|binding| {
                binding
                    .indexed_through_blob_publication()
                    .is_some_and(|publication| removed(publication.record()))
            });
    if invalidates_directory_watermark != directory_replacement.is_some() {
        return false;
    }
    let latest = source
        .root
        .latest_blob_publication()
        .filter(|binding| !removed(binding.record()));
    let directory = source.root.derived_family_directory().filter(|binding| {
        !removed(binding.directory_record())
            && binding
                .indexed_through_blob_publication()
                .is_none_or(|publication| !removed(publication.record()))
    });
    let directory = match directory_replacement {
        None => directory,
        Some(proof) => {
            if source.root != proof.source_root()
                || result.root.generation() != proof.candidate_generation()
                || source.root.derived_family_directory() != Some(proof.previous())
                || source.routes.binary_search_by_key(&proof.source_route().record(), |route| route.record())
                    .ok().is_none_or(|index| source.routes[index] != proof.source_route())
                || projected.binary_search_by_key(&proof.next_route().record(), |route| route.record())
                    .ok().is_none_or(|index| projected[index] != proof.next_route())
                || result.routes.binary_search_by_key(&proof.next_route().record(), |route| route.record())
                    .ok().is_none_or(|index| result.routes[index] != proof.next_route())
                // The replacement belongs to this drop: its descriptor is a
                // record this transition publishes, and a head effect, when
                // the drop has one, names the same operation and descriptor.
                || projected.binary_search_by_key(&proof.descriptor_record(), |route| route.record())
                    .is_err()
                || head_replay.is_some_and(|head| {
                    let worth_store_physical_format::ReleaseCustodyHeadMutationV1::Upsert { next, .. } = head.effect().mutation() else { return true; };
                    head.operation() != proof.operation()
                        || next.descriptor_record() != proof.descriptor_record()
                })
            {
                return false;
            }
            Some(proof.next())
        }
    };
    let quarantine = source
        .root
        .latest_blob_quarantine()
        .filter(|record| !removed(*record));
    let head_matches = match head_replay {
        None => {
            source.root.release_custody_head_root() == result.root.release_custody_head_root()
                && source.root.next_release_custody_head_block()
                    == result.root.next_release_custody_head_block()
        }
        Some(replay) => {
            let effect = replay.effect();
            let worth_store_physical_format::ReleaseCustodyHeadMutationV1::Upsert { next, .. } =
                effect.mutation()
            else {
                return false;
            };
            source.root.release_custody_head_root() == effect.source_root()
                && source.root.next_release_custody_head_block() == effect.source_next_block()
                && source.root.generation() == next.source_root_generation()
                && source.root.tree_identity() == effect.tree_identity()
                && result.root.release_custody_head_root() == Some(replay.result_root())
                && result.root.next_release_custody_head_block() == replay.result_next_block()
                && projected
                    .binary_search_by_key(&next.descriptor_record(), |route| route.record())
                    .is_ok()
        }
    };
    source.root.generation().checked_add(1) == Some(result.root.generation())
        && head_matches
        && source.root.tree_identity() == result.root.tree_identity()
        && result.root.requires_maintenance_protocol()
        && result.root.latest_blob_publication() == latest
        && result.root.derived_family_directory() == directory
        && result.root.latest_blob_quarantine() == quarantine
        && result.root.tier_epoch_anchor() == source.root.tier_epoch_anchor()
        && result.root.last_inline_record() == source.root.last_inline_record()
        && result.root.last_inline_segment() == source.root.last_inline_segment()
        && result.root.node_capacity() == source.root.node_capacity()
        && result.root.next_block() >= source.root.next_block()
        && result.root.next_segment_block() >= source.root.next_segment_block()
        && result.free.next_block() >= source.free.next_block()
}
