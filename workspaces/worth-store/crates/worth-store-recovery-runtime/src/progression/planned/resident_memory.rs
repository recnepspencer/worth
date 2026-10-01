//! One allocation window for physical recovery planning. Observation and image
//! construction carry the same live storage and peak; neither resets admission.

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PlanningMemoryDenial {
    RecoveryMemoryBytes {
        observed: u64,
    },
    Allocation {
        requested_bytes: u64,
        cause: std::collections::TryReserveError,
    },
}

pub(crate) struct PlanningResidentAllowance {
    used: u64,
    maximum: u64,
    peak: u64,
}

impl PlanningResidentAllowance {
    pub(crate) fn new(retained: u64, maximum: u64) -> Result<Self, PlanningMemoryDenial> {
        let mut window = Self {
            used: 0,
            maximum,
            peak: 0,
        };
        window.retain(retained)?;
        Ok(window)
    }

    pub(crate) const fn used(&self) -> u64 {
        self.used
    }

    pub(crate) const fn maximum(&self) -> u64 {
        self.maximum
    }

    pub(crate) const fn remaining(&self) -> u64 {
        self.maximum - self.used
    }

    pub(crate) fn retain(&mut self, bytes: u64) -> Result<(), PlanningMemoryDenial> {
        self.transient(bytes)?;
        self.used = self.used.checked_add(bytes).ok_or_else(Self::overflow)?;
        Ok(())
    }

    pub(crate) fn transient(&mut self, bytes: u64) -> Result<(), PlanningMemoryDenial> {
        let observed = self.used.checked_add(bytes).ok_or_else(Self::overflow)?;
        if observed > self.maximum {
            return Err(PlanningMemoryDenial::RecoveryMemoryBytes { observed });
        }
        self.peak = self.peak.max(observed);
        Ok(())
    }

    pub(crate) fn release(&mut self, bytes: u64) {
        self.used = self
            .used
            .checked_sub(bytes)
            .expect("only charged planning backing is released");
    }

    pub(crate) fn reserve<T>(&mut self, count: usize) -> Result<Vec<T>, PlanningMemoryDenial> {
        let requested = Self::slot_bytes::<T>(count)?;
        self.transient(requested)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|cause| PlanningMemoryDenial::Allocation {
                requested_bytes: requested,
                cause,
            })?;
        self.retain(Self::vector_bytes(&values)?)?;
        Ok(values)
    }

    /// Admit the entire new backing while the old allocation is still live.
    /// Callers must grow before push/extend; only actual capacity growth stays
    /// charged. A failed reservation leaves the existing vector untouched.
    pub(crate) fn grow<T>(
        &mut self,
        values: &mut Vec<T>,
        additional: usize,
    ) -> Result<(), PlanningMemoryDenial> {
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or_else(Self::overflow)?;
        if needed <= values.capacity() {
            return Ok(());
        }
        let capacity = needed.max(
            values
                .capacity()
                .checked_mul(2)
                .ok_or_else(Self::overflow)?,
        );
        let requested = Self::slot_bytes::<T>(capacity)?;
        self.transient(requested)?;
        let old_bytes = Self::vector_bytes(values)?;
        values
            .try_reserve_exact(capacity - values.len())
            .map_err(|cause| PlanningMemoryDenial::Allocation {
                requested_bytes: requested,
                cause,
            })?;
        self.retain(
            Self::vector_bytes(values)?
                .checked_sub(old_bytes)
                .ok_or_else(Self::overflow)?,
        )
    }

    pub(crate) fn into_box<T>(&mut self, values: Vec<T>) -> Result<Box<[T]>, PlanningMemoryDenial> {
        let capacity_bytes = Self::vector_bytes(&values)?;
        let length_bytes = Self::slot_bytes::<T>(values.len())?;
        // A shrinking conversion may allocate before releasing the old Vec.
        self.transient(length_bytes)?;
        let boxed = values.into_boxed_slice();
        self.release(capacity_bytes);
        self.retain(length_bytes)?;
        Ok(boxed)
    }

    pub(crate) fn clone_owned<T: Clone>(
        &mut self,
        value: &T,
        heap_bytes: u64,
    ) -> Result<T, PlanningMemoryDenial> {
        self.retain(heap_bytes)?;
        Ok(value.clone())
    }

    pub(crate) const fn peak(&self) -> u64 {
        self.peak
    }

    pub(crate) fn slot_bytes<T>(count: usize) -> Result<u64, PlanningMemoryDenial> {
        u64::try_from(count)
            .ok()
            .and_then(|count| count.checked_mul(std::mem::size_of::<T>() as u64))
            .ok_or_else(Self::overflow)
    }

    pub(crate) fn vector_bytes<T>(values: &Vec<T>) -> Result<u64, PlanningMemoryDenial> {
        Self::slot_bytes::<T>(values.capacity())
    }

    fn overflow() -> PlanningMemoryDenial {
        PlanningMemoryDenial::RecoveryMemoryBytes { observed: u64::MAX }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn growth_admits_old_and_new_backing_before_mutating_the_vector() {
        let mut denied = PlanningResidentAllowance::new(20, 31).unwrap();
        let mut values = denied.reserve::<u8>(4).unwrap();
        values.extend_from_slice(&[1, 2, 3, 4]);
        assert_eq!(
            denied.grow(&mut values, 1),
            Err(PlanningMemoryDenial::RecoveryMemoryBytes { observed: 32 },)
        );
        assert_eq!(
            (values.as_slice(), values.capacity()),
            (&[1, 2, 3, 4][..], 4)
        );
        assert_eq!(denied.used(), 24);

        let mut admitted = PlanningResidentAllowance::new(20, 32).unwrap();
        let mut values = admitted.reserve::<u8>(4).unwrap();
        values.extend_from_slice(&[1, 2, 3, 4]);
        admitted.grow(&mut values, 1).unwrap();
        values.push(5);
        assert_eq!(
            (admitted.used(), admitted.peak(), admitted.remaining()),
            (28, 32, 4)
        );
        let backing = PlanningResidentAllowance::vector_bytes(&values).unwrap();
        drop(values);
        admitted.release(backing);
        assert_eq!(admitted.used(), 20);
    }

    #[test]
    fn carried_peak_survives_scratch_drop_and_is_not_a_fresh_budget() {
        let mut allowance = PlanningResidentAllowance::new(80, 100).unwrap();
        let scratch = allowance.reserve::<u8>(20).unwrap();
        drop(scratch);
        allowance.release(20);
        assert_eq!((allowance.used(), allowance.peak()), (80, 100));
        assert!(matches!(
            allowance.reserve::<u8>(21),
            Err(PlanningMemoryDenial::RecoveryMemoryBytes { observed: 101 },)
        ));
    }

    #[test]
    fn allocator_capacity_rejection_preserves_cause_without_fabricating_budget_exhaustion() {
        // Vec rejects sizes beyond isize::MAX before calling the allocator.
        // This exercises the real reservation failure without requesting OOM.
        let mut window = PlanningResidentAllowance::new(0, u64::MAX).unwrap();
        let denial = window.reserve::<u8>(usize::MAX).unwrap_err();
        let PlanningMemoryDenial::Allocation {
            requested_bytes,
            cause,
        } = denial
        else {
            panic!("allocator capacity rejection is not a declared-budget crossing")
        };
        assert_eq!(requested_bytes, usize::MAX as u64);
        assert_eq!(
            cause,
            Vec::<u8>::new().try_reserve_exact(usize::MAX).unwrap_err()
        );
    }
}
