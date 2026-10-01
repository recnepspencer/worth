use super::{free_ranges::ArenaFreeRanges, ArenaAllocationDenial, ExtentArenaCapacity};
use std::collections::BTreeMap;
use worth_store_physical_format::{
    arena_tier_at_epoch, ExtentArenaId, ExtentArenaRange, PhysicalTierClass,
};
mod recovered_claim;

/// Reconstructed published free space and separately owned in-flight claims.
/// Only a durable root transition may add a released range to this owner.
pub(in crate::physical_runtime::record_serving) struct ExtentArenaAllocationOwner {
    free: ArenaFreeRanges,
    reservations: BTreeMap<u64, ExtentArenaRange>,
    next_reservation: u64,
    next_arena: u64,
    tier_epoch_start: Option<u64>,
    tier_epoch_pending: bool,
    alignment: u64,
    capacity: ExtentArenaCapacity,
    maximum_ranges: usize,
    // Declared last so index storage is destroyed before its capacity charge.
    _capacity_charge: Option<worth_store_buffer_pool::ForegroundWriteAllocationGrant>,
}

impl ExtentArenaAllocationOwner {
    pub(in crate::physical_runtime::record_serving) fn new(
        capacity: ExtentArenaCapacity,
        alignment: u64,
        maximum_ranges: usize,
        next_arena: u64,
    ) -> Result<Self, ArenaAllocationDenial> {
        Self::new_tiered(capacity, alignment, maximum_ranges, next_arena, None)
    }

    pub(in crate::physical_runtime::record_serving) fn new_tiered(
        capacity: ExtentArenaCapacity,
        alignment: u64,
        maximum_ranges: usize,
        next_arena: u64,
        tier_epoch_start: Option<u64>,
    ) -> Result<Self, ArenaAllocationDenial> {
        if !alignment.is_power_of_two()
            || !capacity.get().is_multiple_of(alignment)
            || maximum_ranges == 0
            || next_arena == 0
            || tier_epoch_start.is_some_and(|epoch| epoch == 0 || epoch > next_arena)
        {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        Ok(Self {
            free: ArenaFreeRanges::new(tier_epoch_start),
            reservations: BTreeMap::new(),
            next_reservation: 1,
            next_arena,
            tier_epoch_start,
            tier_epoch_pending: false,
            alignment,
            capacity,
            maximum_ranges,
            _capacity_charge: None,
        })
    }

    pub(in crate::physical_runtime::record_serving) fn retain_capacity_charge(
        &mut self,
        charge: worth_store_buffer_pool::ForegroundWriteAllocationGrant,
    ) {
        self._capacity_charge = Some(charge);
    }

    pub(in crate::physical_runtime::record_serving) fn restore_free_range(
        &mut self,
        range: ExtentArenaRange,
    ) -> Result<(), ArenaAllocationDenial> {
        self.require_no_tier_transition()?;
        self.validate_range(range)?;
        if range.arena().get() >= self.next_arena {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        self.preflight_release(range)?;
        self.free.insert(range)?;
        Ok(())
    }

    pub(in crate::physical_runtime::record_serving) const fn alignment(&self) -> u64 {
        self.alignment
    }

    pub(in crate::physical_runtime::record_serving) const fn capacity(
        &self,
    ) -> ExtentArenaCapacity {
        self.capacity
    }

    pub(in crate::physical_runtime::record_serving) fn reserve(
        &mut self,
        bytes: u64,
    ) -> Result<(u64, ExtentArenaRange), ArenaAllocationDenial> {
        self.reserve_in_tier(bytes, PhysicalTierClass::Primary)
    }

    pub(in crate::physical_runtime::record_serving) fn reserve_in_tier(
        &mut self,
        bytes: u64,
        tier: PhysicalTierClass,
    ) -> Result<(u64, ExtentArenaRange), ArenaAllocationDenial> {
        self.require_no_tier_transition()?;
        if bytes == 0 || bytes > self.capacity.get() || !bytes.is_multiple_of(self.alignment) {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        if self.tier_epoch_start.is_none() && tier != PhysicalTierClass::Primary {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        let token = self.next_reservation;
        let next = token
            .checked_add(1)
            .ok_or(ArenaAllocationDenial::Capacity)?;
        let range = match self.free.select_best_fit(bytes, tier) {
            Some(selected) => {
                // Taking the free entry removes one index entry; the new claim
                // replaces it, with one additional entry only for a split.
                let required =
                    self.required_ranges(self.free.len(), usize::from(selected.length() > bytes))?;
                self.require_range_capacity(required)?;
                self.free.take_selected(selected, bytes)
            }
            None => {
                let first = self.next_arena;
                let arena_number = (first..=first.saturating_add(2))
                    .find(|id| {
                        ExtentArenaId::new(*id).is_some_and(|arena| {
                            arena_tier_at_epoch(self.tier_epoch_start, arena) == tier
                        })
                    })
                    .ok_or(ArenaAllocationDenial::Capacity)?;
                let next_arena = arena_number
                    .checked_add(1)
                    .ok_or(ArenaAllocationDenial::Capacity)?;
                let added_free = usize::try_from(arena_number - first)
                    .map_err(|_| ArenaAllocationDenial::Capacity)?
                    .checked_add(usize::from(bytes < self.capacity.get()))
                    .ok_or(ArenaAllocationDenial::Capacity)?;
                let required = self.required_ranges(
                    self.free.len(),
                    added_free
                        .checked_add(1)
                        .ok_or(ArenaAllocationDenial::Capacity)?,
                )?;
                self.require_range_capacity(required)?;
                for id in first..arena_number {
                    let skipped = ExtentArenaId::new(id).ok_or(ArenaAllocationDenial::Capacity)?;
                    self.free.insert(
                        ExtentArenaRange::new(skipped, 0, self.capacity.get())
                            .ok_or(ArenaAllocationDenial::InvalidGeometry)?,
                    )?;
                }
                let arena =
                    ExtentArenaId::new(arena_number).ok_or(ArenaAllocationDenial::Capacity)?;
                if bytes < self.capacity.get() {
                    self.free.insert(
                        ExtentArenaRange::new(arena, bytes, self.capacity.get() - bytes)
                            .ok_or(ArenaAllocationDenial::InvalidGeometry)?,
                    )?;
                }
                self.next_arena = next_arena;
                ExtentArenaRange::new(arena, 0, bytes)
                    .ok_or(ArenaAllocationDenial::InvalidGeometry)?
            }
        };
        self.next_reservation = next;
        self.reservations.insert(token, range);
        Ok((token, range))
    }

    pub(in crate::physical_runtime::record_serving) fn cancel(
        &mut self,
        token: u64,
    ) -> Result<(), ArenaAllocationDenial> {
        let range = self
            .reservations
            .remove(&token)
            .ok_or(ArenaAllocationDenial::StaleReservation)?;
        self.free.insert(range)
    }

    pub(in crate::physical_runtime::record_serving) fn published(
        &mut self,
        token: u64,
    ) -> Result<ExtentArenaRange, ArenaAllocationDenial> {
        self.reservations
            .remove(&token)
            .ok_or(ArenaAllocationDenial::StaleReservation)
    }

    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime::record_serving) fn certification_has_exact_claim(
        &self,
        token: u64,
        range: ExtentArenaRange,
    ) -> bool {
        self.reservations.get(&token) == Some(&range)
    }

    pub(in crate::physical_runtime::record_serving) fn admit_durable_release(
        &mut self,
        range: ExtentArenaRange,
    ) -> Result<(), ArenaAllocationDenial> {
        self.preflight_release(range)?;
        self.free.insert(range)
    }

    pub(in crate::physical_runtime::record_serving) fn preflight_release(
        &self,
        range: ExtentArenaRange,
    ) -> Result<(), ArenaAllocationDenial> {
        self.require_no_tier_transition()?;
        self.validate_range(range)?;
        let required = self.required_ranges(self.free.entries_after_insert(range)?, 0)?;
        self.require_range_capacity(required)
    }

    pub(in crate::physical_runtime::record_serving) fn release_touches_free(
        &self,
        range: ExtentArenaRange,
    ) -> bool {
        self.free.touches(range)
    }

    pub(in crate::physical_runtime::record_serving) fn contains_released(
        &self,
        range: ExtentArenaRange,
    ) -> bool {
        self.free.contains(range)
    }

    pub(in crate::physical_runtime::record_serving) fn begin_evacuation(
        &mut self,
        arena: ExtentArenaId,
    ) -> Result<(), ArenaAllocationDenial> {
        self.require_no_tier_transition()?;
        if arena.get() >= self.next_arena
            || self
                .reservations
                .values()
                .any(|range| range.arena() == arena)
        {
            return Err(ArenaAllocationDenial::EvacuationBusy);
        }
        self.free.exclude(arena)
    }

    pub(in crate::physical_runtime::record_serving) fn sparse_candidate(
        &self,
        published_next: u64,
        threshold: super::ArenaEvacuationThreshold,
    ) -> Option<ExtentArenaId> {
        self.free
            .sparse_candidate(self.capacity.get(), published_next, threshold)
    }

    pub(super) fn cancel_evacuation(&mut self, arena: ExtentArenaId) {
        self.free.include(arena);
    }

    pub(in crate::physical_runtime::record_serving) fn forget_evacuated(
        &mut self,
        arena: ExtentArenaId,
    ) {
        self.free.forget(arena);
    }

    fn validate_range(&self, range: ExtentArenaRange) -> Result<(), ArenaAllocationDenial> {
        if range.end() > self.capacity.get()
            || !range.offset().is_multiple_of(self.alignment)
            || !range.length().is_multiple_of(self.alignment)
        {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        Ok(())
    }

    fn required_ranges(
        &self,
        free_entries: usize,
        additional: usize,
    ) -> Result<usize, ArenaAllocationDenial> {
        free_entries
            .checked_add(self.reservations.len())
            .and_then(|count| count.checked_add(additional))
            .ok_or(ArenaAllocationDenial::Capacity)
    }

    fn require_range_capacity(&self, required: usize) -> Result<(), ArenaAllocationDenial> {
        if required > self.maximum_ranges {
            Err(ArenaAllocationDenial::RangeBudget {
                required,
                maximum: self.maximum_ranges,
            })
        } else {
            Ok(())
        }
    }

    fn require_no_tier_transition(&self) -> Result<(), ArenaAllocationDenial> {
        if self.tier_epoch_pending {
            Err(ArenaAllocationDenial::EvacuationBusy)
        } else {
            Ok(())
        }
    }

    pub(super) fn begin_tier_epoch_activation(
        &mut self,
        published_next_arena: u64,
    ) -> Result<u64, ArenaAllocationDenial> {
        self.require_no_tier_transition()?;
        if self.tier_epoch_start.is_some()
            || self.next_arena != published_next_arena
            || !self.reservations.is_empty()
            || self.free.has_exclusion()
        {
            return Err(ArenaAllocationDenial::EvacuationBusy);
        }
        self.tier_epoch_pending = true;
        Ok(published_next_arena)
    }

    pub(super) fn complete_tier_epoch_activation(
        &mut self,
        epoch: u64,
    ) -> Result<(), ArenaAllocationDenial> {
        if !self.tier_epoch_pending
            || self.tier_epoch_start.is_some()
            || epoch != self.next_arena
            || !self.reservations.is_empty()
        {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        self.free.activate_tier_epoch(epoch)?;
        self.tier_epoch_start = Some(epoch);
        self.tier_epoch_pending = false;
        Ok(())
    }

    pub(super) fn abort_tier_epoch_activation_before_effect(&mut self) {
        if self.tier_epoch_start.is_none() {
            self.tier_epoch_pending = false;
        }
    }
}
