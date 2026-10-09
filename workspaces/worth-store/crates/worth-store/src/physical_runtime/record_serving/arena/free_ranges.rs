use super::ArenaAllocationDenial;
use std::collections::{BTreeMap, BTreeSet};
use worth_store_physical_format::{
    arena_tier_at_epoch, ExtentArenaId, ExtentArenaRange, PhysicalTierClass,
};

/// Two indexes over the same admitted free ranges. The address index owns
/// coalescing; the size index selects best fit without a media read or scan.
pub(super) struct ArenaFreeRanges {
    by_address: BTreeMap<(ExtentArenaId, u64), u64>,
    by_tier_size: [BTreeSet<(u64, ExtentArenaId, u64)>; 3],
    tier_epoch_start: Option<u64>,
    excluded: Option<ExtentArenaId>,
}

impl ArenaFreeRanges {
    pub(super) fn new(tier_epoch_start: Option<u64>) -> Self {
        Self {
            by_address: BTreeMap::new(),
            by_tier_size: std::array::from_fn(|_| BTreeSet::new()),
            tier_epoch_start,
            excluded: None,
        }
    }

    fn tier_index(&self, arena: ExtentArenaId) -> usize {
        match arena_tier_at_epoch(self.tier_epoch_start, arena) {
            PhysicalTierClass::Primary => 0,
            PhysicalTierClass::Hot => 1,
            PhysicalTierClass::Cold => 2,
        }
    }

    pub(super) fn len(&self) -> usize {
        self.by_address.len()
    }

    #[cfg(any(test, feature = "certification-test-authority"))]
    pub(super) fn has_exclusion(&self) -> bool {
        self.excluded.is_some()
    }

    /// At the epoch frontier every existing arena remains Primary, so the
    /// size indexes require no migration and future insertions use the epoch.
    #[cfg(any(test, feature = "certification-test-authority"))]
    pub(super) fn activate_tier_epoch(&mut self, epoch: u64) -> Result<(), ArenaAllocationDenial> {
        if self.tier_epoch_start.is_some()
            || self
                .by_address
                .keys()
                .any(|(arena, _)| arena.get() >= epoch)
        {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        self.tier_epoch_start = Some(epoch);
        Ok(())
    }

    /// Exact final address-entry count; coalescing releases can reduce a full
    /// index and must not be denied by a worst-case insertion allowance.
    pub(super) fn entries_after_insert(
        &self,
        range: ExtentArenaRange,
    ) -> Result<usize, ArenaAllocationDenial> {
        let arena = range.arena();
        let mut count = self.by_address.len() + 1;
        if let Some((&(id, offset), &length)) = self
            .by_address
            .range(..=(arena, range.offset()))
            .next_back()
        {
            if id == arena {
                if offset + length > range.offset() {
                    return Err(ArenaAllocationDenial::Overlap);
                }
                if offset + length == range.offset() {
                    count -= 1;
                }
            }
        }
        if let Some((&(id, offset), _)) = self.by_address.range((arena, range.offset())..).next() {
            if id == arena {
                if offset < range.end() {
                    return Err(ArenaAllocationDenial::Overlap);
                }
                if offset == range.end() {
                    count -= 1;
                }
            }
        }
        Ok(count)
    }

    pub(super) fn touches(&self, range: ExtentArenaRange) -> bool {
        let arena = range.arena();
        self.by_address
            .range(..=(arena, range.offset()))
            .next_back()
            .is_some_and(|(&(id, offset), &length)| {
                id == arena && offset + length == range.offset()
            })
            || self.by_address.contains_key(&(arena, range.end()))
    }

    pub(super) fn contains(&self, range: ExtentArenaRange) -> bool {
        self.by_address
            .range(..=(range.arena(), range.offset()))
            .next_back()
            .is_some_and(|(&(arena, offset), &length)| {
                arena == range.arena() && offset <= range.offset() && offset + length >= range.end()
            })
    }

    pub(super) fn sparse_candidate(
        &self,
        capacity: u64,
        published_next: u64,
        threshold: super::ArenaEvacuationThreshold,
    ) -> Option<ExtentArenaId> {
        let mut arena = None;
        let mut free = 0_u64;
        for (&(id, _), &length) in &self.by_address {
            if Some(id) != arena {
                if let Some(previous) = arena {
                    if threshold.requires_evacuation(capacity.saturating_sub(free), capacity) {
                        return Some(previous);
                    }
                }
                if id.get().saturating_add(1) >= published_next {
                    return None;
                }
                arena = Some(id);
                free = 0;
            }
            free = free.saturating_add(length);
        }
        arena.filter(|_| threshold.requires_evacuation(capacity.saturating_sub(free), capacity))
    }

    pub(super) fn containing(&self, range: ExtentArenaRange) -> Option<ExtentArenaRange> {
        let (&(arena, offset), &length) = self
            .by_address
            .range(..=(range.arena(), range.offset()))
            .next_back()?;
        (arena == range.arena() && offset + length >= range.end())
            .then(|| ExtentArenaRange::new(arena, offset, length))
            .flatten()
    }

    /// Exact WAL-identified claim, never a new best-fit decision during reopen.
    pub(super) fn take_exact(
        &mut self,
        range: ExtentArenaRange,
    ) -> Result<(), ArenaAllocationDenial> {
        let enclosing = self
            .containing(range)
            .ok_or(ArenaAllocationDenial::Overlap)?;
        self.remove(enclosing.arena(), enclosing.offset(), enclosing.length());
        if enclosing.offset() < range.offset() {
            self.insert(
                ExtentArenaRange::new(
                    range.arena(),
                    enclosing.offset(),
                    range.offset() - enclosing.offset(),
                )
                .unwrap(),
            )?;
        }
        if range.end() < enclosing.end() {
            self.insert(
                ExtentArenaRange::new(range.arena(), range.end(), enclosing.end() - range.end())
                    .unwrap(),
            )?;
        }
        Ok(())
    }

    pub(super) fn insert(&mut self, range: ExtentArenaRange) -> Result<(), ArenaAllocationDenial> {
        let arena = range.arena();
        let mut start = range.offset();
        let mut end = range.end();
        let previous = self
            .by_address
            .range(..=(arena, start))
            .next_back()
            .map(|(&(id, offset), &length)| (id, offset, length));
        let next = self
            .by_address
            .range((arena, start)..)
            .next()
            .map(|(&(id, offset), &length)| (id, offset, length));
        if let Some((id, offset, length)) = previous {
            if id == arena && offset + length > start {
                return Err(ArenaAllocationDenial::Overlap);
            }
        }
        if let Some((id, offset, _)) = next {
            if id == arena && offset < end {
                return Err(ArenaAllocationDenial::Overlap);
            }
        }
        if let Some((id, offset, length)) = previous {
            if id == arena && offset + length == start {
                self.remove(id, offset, length);
                start = offset;
            }
        }
        if let Some((id, offset, length)) = next {
            if id == arena && offset == end {
                self.remove(id, offset, length);
                end = offset + length;
            }
        }
        self.by_address.insert((arena, start), end - start);
        if self.excluded != Some(arena) {
            let tier_index = self.tier_index(arena);
            self.by_tier_size[tier_index].insert((end - start, arena, start));
        }
        Ok(())
    }

    pub(super) fn select_best_fit(
        &self,
        bytes: u64,
        tier: PhysicalTierClass,
    ) -> Option<ExtentArenaRange> {
        let minimum = ExtentArenaId::new(1)?;
        let index = match tier {
            PhysicalTierClass::Primary => 0,
            PhysicalTierClass::Hot => 1,
            PhysicalTierClass::Cold => 2,
        };
        let (length, arena, offset) = self.by_tier_size[index]
            .range((bytes, minimum, 0)..)
            .next()
            .copied()?;
        ExtentArenaRange::new(arena, offset, length)
    }

    pub(super) fn take_selected(
        &mut self,
        selected: ExtentArenaRange,
        bytes: u64,
    ) -> ExtentArenaRange {
        let arena = selected.arena();
        let offset = selected.offset();
        assert!(
            selected.length() >= bytes,
            "selected free range contains claim"
        );
        let claim = ExtentArenaRange::new(arena, offset, bytes)
            .expect("selected free range contains the admitted aligned reservation");
        self.remove(arena, offset, selected.length());
        if selected.length() > bytes {
            self.by_address
                .insert((arena, offset + bytes), selected.length() - bytes);
            let index = self.tier_index(arena);
            self.by_tier_size[index].insert((selected.length() - bytes, arena, offset + bytes));
        }
        claim
    }

    fn remove(&mut self, arena: ExtentArenaId, offset: u64, length: u64) {
        self.by_address.remove(&(arena, offset));
        let tier_index = self.tier_index(arena);
        self.by_tier_size[tier_index].remove(&(length, arena, offset));
    }

    pub(super) fn exclude(&mut self, arena: ExtentArenaId) -> Result<(), ArenaAllocationDenial> {
        if self.excluded.is_some() {
            return Err(ArenaAllocationDenial::EvacuationBusy);
        }
        self.excluded = Some(arena);
        let tier_index = self.tier_index(arena);
        for (&(id, offset), &length) in self.by_address.range((arena, 0)..=(arena, u64::MAX)) {
            self.by_tier_size[tier_index].remove(&(length, id, offset));
        }
        Ok(())
    }

    pub(super) fn include(&mut self, arena: ExtentArenaId) {
        assert_eq!(
            self.excluded,
            Some(arena),
            "only the exclusion lease may restore allocation"
        );
        let tier_index = self.tier_index(arena);
        for (&(id, offset), &length) in self.by_address.range((arena, 0)..=(arena, u64::MAX)) {
            self.by_tier_size[tier_index].insert((length, id, offset));
        }
        self.excluded = None;
    }

    pub(super) fn forget(&mut self, arena: ExtentArenaId) {
        assert_eq!(
            self.excluded,
            Some(arena),
            "durable removal requires exclusive exclusion"
        );
        self.by_address.retain(|(id, _), _| *id != arena);
        self.excluded = None;
    }
}
