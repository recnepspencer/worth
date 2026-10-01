use super::*;

pub(super) fn routes_match(
    source: ReleasedInventoryView<'_>,
    result: ReleasedInventoryView<'_>,
    projection: &PersistedPhysicalRecoveryProjection,
) -> Result<bool, OrdinaryRootStepDenial> {
    use OrdinaryRootStepDenial as Denial;
    let dropped = match projection.operation() {
        Semantic::DerivedDirectory {
            retirement: Some(retirement),
            ..
        } => retirement.dropped_records(),
        _ => &[],
    };
    if dropped.windows(2).any(|pair| pair[0] >= pair[1])
        || dropped.iter().any(|record| {
            source
                .routes
                .binary_search_by_key(record, |route| route.record())
                .is_err()
        })
    {
        return Ok(false);
    }
    let mut routes = source
        .routes
        .iter()
        .copied()
        .filter(|route| dropped.binary_search(&route.record()).is_err())
        .map(|route| (route.record(), route))
        .collect::<BTreeMap<_, _>>();
    if routes.len() != source.routes.len().saturating_sub(dropped.len()) {
        return Ok(false);
    }
    for placement in projection.placements() {
        if dropped.binary_search(&placement.record()).is_ok() {
            let CurrentPhysicalRecordPlacement::Inline(retired) = placement else {
                return Ok(false);
            };
            let matching_source = source
                .routes
                .binary_search_by_key(&placement.record(), |route| route.record())
                .ok()
                .is_some_and(|index| retired_witness_matches(source.routes[index], *retired));
            let shared_rewrite_page = projection.placements().iter().any(|candidate| {
                dropped.binary_search(&candidate.record()).is_err()
                    && matches!(candidate, CurrentPhysicalRecordPlacement::Inline(live)
                        if live.page_cell() == retired.page_cell())
            });
            if !matching_source || !shared_rewrite_page {
                return Ok(false);
            }
            continue;
        }
        if routes
            .insert(placement.record(), *placement)
            .is_some_and(|prior| matches!(prior, CurrentPhysicalRecordPlacement::Extent(_)))
        {
            return Ok(false);
        }
    }
    let mut expected = Vec::new();
    expected
        .try_reserve_exact(routes.len())
        .map_err(|_| Denial::BoundExceeded)?;
    expected.extend(routes.into_values());
    Ok(expected == result.routes)
}

pub(super) fn retired_witness_matches(
    source: CurrentPhysicalRecordPlacement,
    projected: worth_store_physical_format::DurableInlineRecordPlacement,
) -> bool {
    let CurrentPhysicalRecordPlacement::Inline(source) = source else {
        return false;
    };
    // C.9 may carry a dropped record into the newly written page while the
    // successor root unroutes it. Stable slot/content identity must still
    // match the selected source; only the page/segment generation may advance.
    source.record() == projected.record()
        && source.segment() == projected.segment()
        && source.page() == projected.page()
        && source.slot() == projected.slot()
        && source.slot_generation() == projected.slot_generation()
        && source.segment_page_capacity() == projected.segment_page_capacity()
        && source.payload_bytes() == projected.payload_bytes()
        && source.route_metadata() == projected.route_metadata()
        && projected.segment_generation() >= source.segment_generation()
        && projected.page_generation() >= source.page_generation()
}

pub(super) fn segments_match(
    source: ReleasedInventoryView<'_>,
    result: ReleasedInventoryView<'_>,
    projection: &PersistedPhysicalRecoveryProjection,
) -> Result<bool, OrdinaryRootStepDenial> {
    let mut pages = source
        .segments
        .iter()
        .copied()
        .map(|entry| (SegmentPageKey::from(entry), entry))
        .collect::<BTreeMap<_, RecordSegmentPageManifestEntry>>();
    if pages.len() != source.segments.len() {
        return Ok(false);
    }
    for update in projection.segment_updates() {
        pages.insert(SegmentPageKey::from(*update), *update);
    }
    Ok(pages.len() == result.segments.len()
        && pages.into_values().eq(result.segments.iter().copied()))
}

pub(super) fn free_matches(
    source: ReleasedInventoryView<'_>,
    result: ReleasedInventoryView<'_>,
    projection: &PersistedPhysicalRecoveryProjection,
    limit: u64,
) -> Result<bool, OrdinaryRootStepDenial> {
    use OrdinaryRootStepDenial as Denial;
    let mut free = source
        .free_entries
        .iter()
        .copied()
        .map(|entry| (FreeSpaceKey::from(entry), entry))
        .collect::<BTreeMap<_, _>>();
    if free.len() != source.free_entries.len() {
        return Ok(false);
    }
    for allocation in projection.root_state().inline_allocations() {
        let key = FreeSpaceKey::inline(allocation.segment().segment_id().get())
            .ok_or(Denial::InvalidDelta)?;
        if allocation.used_pages() < allocation.page_capacity() {
            let entry = RecordFreeSpaceManifestEntry::inline_frontier(
                allocation.segment().segment_id().get(),
                u64::from(allocation.used_pages()) + 1,
                u64::from(allocation.page_capacity() - allocation.used_pages()),
                allocation.segment().generation().get(),
            )
            .ok_or(Denial::InvalidDelta)?;
            free.insert(key, entry);
        } else {
            free.remove(&key);
        }
    }
    let mut ranges = projection
        .placements()
        .iter()
        .filter_map(|placement| match placement {
            CurrentPhysicalRecordPlacement::Extent(extent) => Some(extent.arena_range()),
            CurrentPhysicalRecordPlacement::Inline(_) => None,
        })
        .collect::<Vec<_>>();
    ranges.sort_unstable_by_key(|range| (range.arena(), range.offset()));
    let first = source.free.next_arena();
    let next_arena = ranges
        .iter()
        .try_fold(first, |next, range| {
            range
                .arena()
                .get()
                .checked_add(1)
                .map(|candidate| next.max(candidate))
        })
        .ok_or(Denial::InvalidDelta)?;
    if next_arena
        .checked_sub(first)
        .is_none_or(|count| count > limit)
    {
        return Err(Denial::BoundExceeded);
    }
    for id in first..next_arena {
        if free.len() as u64 >= limit {
            return Err(Denial::BoundExceeded);
        }
        let range = ExtentArenaRange::new(
            ExtentArenaId::new(id).ok_or(Denial::InvalidDelta)?,
            0,
            source.free.arena_capacity(),
        )
        .ok_or(Denial::InvalidDelta)?;
        insert(&mut free, range, result.root.generation())?;
    }
    for range in ranges {
        if free.len() as u64 > limit {
            return Err(Denial::BoundExceeded);
        }
        if range.end() > source.free.arena_capacity()
            || range.offset() % source.free.arena_alignment() != 0
            || range.length() % source.free.arena_alignment() != 0
        {
            return Ok(false);
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
            });
        let Some(available) = available else {
            return Ok(false);
        };
        free.remove(&FreeSpaceKey::arena(available.arena(), available.offset()));
        if available.offset() < range.offset() {
            insert(
                &mut free,
                ExtentArenaRange::new(
                    range.arena(),
                    available.offset(),
                    range.offset() - available.offset(),
                )
                .ok_or(Denial::InvalidDelta)?,
                result.root.generation(),
            )?;
        }
        if range.end() < available.end() {
            insert(
                &mut free,
                ExtentArenaRange::new(range.arena(), range.end(), available.end() - range.end())
                    .ok_or(Denial::InvalidDelta)?,
                result.root.generation(),
            )?;
        }
    }
    let next_segment = projection
        .root_state()
        .inline_allocations()
        .iter()
        .try_fold(source.free.next_segment(), |next, allocation| {
            allocation
                .segment()
                .segment_id()
                .get()
                .checked_add(1)
                .map(|candidate| next.max(candidate))
        })
        .ok_or(Denial::InvalidDelta)?;
    let (next_page, next_extent) = result
        .routes
        .iter()
        .try_fold(
            (source.free.next_page(), source.free.next_extent()),
            |(page, extent), route| match route {
                CurrentPhysicalRecordPlacement::Inline(inline) => inline
                    .page()
                    .get()
                    .checked_add(1)
                    .map(|candidate| (page.max(candidate), extent)),
                CurrentPhysicalRecordPlacement::Extent(value) => value
                    .extent()
                    .get()
                    .checked_add(1)
                    .map(|candidate| (page, extent.max(candidate))),
            },
        )
        .ok_or(Denial::InvalidDelta)?;
    Ok(free.len() == result.free_entries.len()
        && free.into_values().eq(result.free_entries.iter().copied())
        && result.free.next_segment() == next_segment
        && result.free.next_page() == next_page
        && result.free.next_extent() == next_extent
        && result.free.next_arena() == next_arena
        && result.free.tree_identity() == source.free.tree_identity()
        && result.free.node_capacity() == projection.root_state().successor_manifest_capacity()
        && result.free.segment_page_capacity() == source.free.segment_page_capacity()
        && result.free.arena_capacity() == source.free.arena_capacity()
        && result.free.arena_alignment() == source.free.arena_alignment()
        && result.free.tier_epoch_start() == source.free.tier_epoch_start())
}

fn insert(
    free: &mut BTreeMap<FreeSpaceKey, RecordFreeSpaceManifestEntry>,
    range: ExtentArenaRange,
    generation: u64,
) -> Result<(), OrdinaryRootStepDenial> {
    let entry = RecordFreeSpaceManifestEntry::arena_range(range, generation)
        .ok_or(OrdinaryRootStepDenial::InvalidDelta)?;
    if free.insert(FreeSpaceKey::from(entry), entry).is_some() {
        return Err(OrdinaryRootStepDenial::InvalidDelta);
    }
    Ok(())
}
