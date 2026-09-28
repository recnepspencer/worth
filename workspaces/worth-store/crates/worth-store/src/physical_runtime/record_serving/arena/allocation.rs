use super::{free_ranges::ArenaFreeRanges, ArenaAllocationDenial, ExtentArenaCapacity};
use std::collections::BTreeMap;
use worth_store_physical_format::{ExtentArenaId, ExtentArenaRange};
mod recovered_claim;

/// Reconstructed published free space and separately owned in-flight claims.
/// Only a durable root transition may add a released range to this owner.
pub(in crate::physical_runtime::record_serving) struct ExtentArenaAllocationOwner {
    free: ArenaFreeRanges,
    reservations: BTreeMap<u64, ExtentArenaRange>,
    next_reservation: u64,
    next_arena: u64,
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
        if !alignment.is_power_of_two()
            || !capacity.get().is_multiple_of(alignment)
            || maximum_ranges == 0
            || next_arena == 0
        {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        Ok(Self {
            free: ArenaFreeRanges::default(),
            reservations: BTreeMap::new(),
            next_reservation: 1,
            next_arena,
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
        if bytes == 0 || bytes > self.capacity.get() || !bytes.is_multiple_of(self.alignment) {
            return Err(ArenaAllocationDenial::InvalidGeometry);
        }
        self.require_budget()?;
        let token = self.next_reservation;
        let next = token
            .checked_add(1)
            .ok_or(ArenaAllocationDenial::Capacity)?;
        let range = match self.free.take_best_fit(bytes) {
            Some(range) => range,
            None => {
                let arena =
                    ExtentArenaId::new(self.next_arena).ok_or(ArenaAllocationDenial::Capacity)?;
                self.next_arena = self
                    .next_arena
                    .checked_add(1)
                    .ok_or(ArenaAllocationDenial::Capacity)?;
                if bytes < self.capacity.get() {
                    self.free.insert(
                        ExtentArenaRange::new(arena, bytes, self.capacity.get() - bytes)
                            .ok_or(ArenaAllocationDenial::InvalidGeometry)?,
                    )?;
                }
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
        self.validate_range(range)?;
        if self.free.entries_after_insert(range)? + self.reservations.len() > self.maximum_ranges {
            return Err(ArenaAllocationDenial::RangeBudget);
        }
        Ok(())
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

    fn require_budget(&self) -> Result<(), ArenaAllocationDenial> {
        if self.free.len() + self.reservations.len() + 2 > self.maximum_ranges {
            return Err(ArenaAllocationDenial::RangeBudget);
        }
        Ok(())
    }
}
