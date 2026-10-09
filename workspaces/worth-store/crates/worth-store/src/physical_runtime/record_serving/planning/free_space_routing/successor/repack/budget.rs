use super::{RecordAppendDenial, RecordAppendError};

fn pressure() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::from_residency(
        worth_store_buffer_pool::PhysicalResidencyDenial::MetadataBudgetExceeded,
    ))
}

pub(super) struct LocalRepackBudget {
    maximum: u64,
    live: u64,
    poisoned: bool,
}

impl LocalRepackBudget {
    pub(super) fn new(maximum: u64, declared_peak: u64) -> Result<Self, RecordAppendError> {
        if declared_peak > maximum {
            return Err(pressure());
        }
        Ok(Self {
            maximum,
            live: 0,
            poisoned: false,
        })
    }

    fn checked_bytes<T>(capacity: usize) -> Result<u64, RecordAppendError> {
        u64::try_from(capacity)
            .ok()
            .and_then(|count| count.checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?))
            .ok_or_else(pressure)
    }

    fn check(&mut self, additional: u64) -> Result<(), RecordAppendError> {
        let allowed = !self.poisoned
            && self
                .live
                .checked_add(additional)
                .is_some_and(|required| required <= self.maximum);
        if !allowed {
            self.poisoned = true;
            return Err(pressure());
        }
        Ok(())
    }

    /// Reserve while the old backing may still coexist with the replacement.
    pub(super) fn reserve_vec<T>(
        &mut self,
        value: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), RecordAppendError> {
        if self.poisoned {
            return Err(pressure());
        }
        let requested_count = value.len().checked_add(additional).ok_or_else(pressure)?;
        if requested_count <= value.capacity() {
            return Ok(());
        }
        let requested = Self::checked_bytes::<T>(requested_count)?;
        self.check(requested)?;
        let old_capacity = value.capacity();
        value.try_reserve_exact(additional).map_err(|cause| {
            self.poisoned = true;
            RecordAppendError::Denied(RecordAppendDenial::PlanningAllocationUnavailable {
                requested,
                cause,
            })
        })?;
        self.reconcile::<T>(old_capacity, value.capacity())
    }

    fn reconcile<T>(
        &mut self,
        old_capacity: usize,
        new_capacity: usize,
    ) -> Result<(), RecordAppendError> {
        let old = Self::checked_bytes::<T>(old_capacity)?;
        let new = Self::checked_bytes::<T>(new_capacity)?;
        // The allocator may have held both backings while growing the Vec.
        // Reconcile the actual new capacity against that overlap before crediting old.
        self.check(new)?;
        self.live = self
            .live
            .checked_sub(old)
            .and_then(|remaining| remaining.checked_add(new))
            .ok_or_else(|| {
                self.poisoned = true;
                pressure()
            })?;
        Ok(())
    }

    /// Format encoding only resizes the same pre-reserved Vec; it cannot grow it.
    pub(super) fn reconcile_transfer(
        &mut self,
        old_capacity: usize,
        new_capacity: usize,
    ) -> Result<(), RecordAppendError> {
        if old_capacity != new_capacity {
            self.poisoned = true;
            return Err(pressure());
        }
        Ok(())
    }

    pub(super) fn source_read_window(
        &mut self,
        page_bytes: u64,
        node_bytes: u64,
    ) -> Result<(), RecordAppendError> {
        self.check(page_bytes.checked_add(node_bytes).ok_or_else(pressure)?)
    }

    pub(super) fn retain<T>(&mut self, value: &Vec<T>) -> Result<(), RecordAppendError> {
        let bytes = Self::checked_bytes::<T>(value.capacity())?;
        self.check(bytes)?;
        self.live = self.live.checked_add(bytes).ok_or_else(pressure)?;
        Ok(())
    }

    /// Call only after the Vec (or the block owning its moved backing) drops.
    pub(super) fn release_capacity<T>(&mut self, capacity: usize) {
        let bytes = Self::checked_bytes::<T>(capacity).expect("admitted backing size");
        self.live = self.live.checked_sub(bytes).expect("charged backing");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocator_failure_preserves_cause_and_poisoned_local_window() {
        let mut budget = LocalRepackBudget::new(u64::MAX, 0).unwrap();
        let mut frame = Vec::<u8>::new();
        let direct_cause = Vec::<u8>::new().try_reserve_exact(usize::MAX).unwrap_err();
        let failure = budget.reserve_vec(&mut frame, usize::MAX).unwrap_err();
        match failure {
            RecordAppendError::Denied(RecordAppendDenial::PlanningAllocationUnavailable {
                requested,
                cause,
            }) => {
                assert_eq!(requested, u64::try_from(usize::MAX).unwrap());
                assert_eq!(cause, direct_cause);
            }
            other => panic!("allocation cause was erased: {other:?}"),
        }
        assert!(budget.reserve_vec(&mut frame, 1).is_err());
    }

    #[test]
    fn actual_growth_overlap_denies_even_when_replacement_alone_fits() {
        let mut budget = LocalRepackBudget::new(u64::MAX, 0).unwrap();
        let mut frame = Vec::<u8>::new();
        budget.reserve_vec(&mut frame, 8).unwrap();
        let old_capacity = frame.capacity();
        let replacement_capacity = old_capacity + 1;
        budget.maximum = u64::try_from(old_capacity + replacement_capacity - 1).unwrap();
        assert!(u64::try_from(replacement_capacity).unwrap() <= budget.maximum);
        assert!(budget
            .reconcile::<u8>(old_capacity, replacement_capacity)
            .is_err());
        assert!(budget.poisoned);
    }
}
