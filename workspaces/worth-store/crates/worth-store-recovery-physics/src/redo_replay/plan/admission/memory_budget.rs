//! Pre-allocation memory budget for physical redo admission.

use worth_store_physical_format::PhysicalRecoveryDecodeStorage;

use super::super::PhysicalRedoPlanningDenial;

pub(in crate::redo_replay) struct RedoAllocationBudget {
    used: u64,
    limit: u64,
}

impl RedoAllocationBudget {
    pub(in crate::redo_replay) const fn new(limit: u64) -> Self {
        Self { used: 0, limit }
    }

    pub(in crate::redo_replay) fn slot_bytes<T>(
        count: usize,
    ) -> Result<u64, PhysicalRedoPlanningDenial> {
        count
            .checked_mul(std::mem::size_of::<T>())
            .filter(|bytes| *bytes <= isize::MAX as usize)
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)
    }

    pub(in crate::redo_replay) fn transient(
        &self,
        bytes: u64,
    ) -> Result<(), PhysicalRedoPlanningDenial> {
        let observed = self
            .used
            .checked_add(bytes)
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
        if observed > self.limit {
            return Err(PhysicalRedoPlanningDenial::RecoveryMemoryLimit {
                observed,
                admitted: self.limit,
            });
        }
        Ok(())
    }

    pub(in crate::redo_replay) fn admit(
        &mut self,
        bytes: u64,
    ) -> Result<(), PhysicalRedoPlanningDenial> {
        self.transient(bytes)?;
        self.used = self
            .used
            .checked_add(bytes)
            .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
        Ok(())
    }

    pub(in crate::redo_replay) fn release(&mut self, bytes: u64) {
        self.used = self.used.saturating_sub(bytes);
    }

    pub(in crate::redo_replay) fn reserve_exact<T>(
        &mut self,
        count: usize,
    ) -> Result<Vec<T>, PhysicalRedoPlanningDenial> {
        let bytes = Self::slot_bytes::<T>(count)?;
        self.transient(bytes)?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| PhysicalRedoPlanningDenial::CounterOverflow)?;
        let actual = Self::slot_bytes::<T>(values.capacity())?;
        if actual > bytes {
            self.transient(actual)?;
        }
        self.admit(actual)?;
        Ok(values)
    }

    pub(in crate::redo_replay) fn push<T>(
        &mut self,
        values: &mut Vec<T>,
        value: T,
    ) -> Result<(), PhysicalRedoPlanningDenial> {
        if values.len() == values.capacity() {
            let next = if values.capacity() == 0 {
                1
            } else {
                values
                    .capacity()
                    .checked_mul(2)
                    .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?
            };
            let new_bytes = Self::slot_bytes::<T>(next)?;
            // Admit old + new backing overlap before growing.
            self.transient(new_bytes)?;
            let old_bytes = Self::slot_bytes::<T>(values.capacity())?;
            values
                .try_reserve_exact(next - values.len())
                .map_err(|_| PhysicalRedoPlanningDenial::CounterOverflow)?;
            let actual_new = Self::slot_bytes::<T>(values.capacity())?;
            let growth = actual_new
                .checked_sub(old_bytes)
                .ok_or(PhysicalRedoPlanningDenial::CounterOverflow)?;
            self.admit(growth)?;
        }
        values.push(value);
        Ok(())
    }

    pub(in crate::redo_replay) fn into_boxed_slice<T>(
        &mut self,
        values: Vec<T>,
    ) -> Result<Box<[T]>, PhysicalRedoPlanningDenial> {
        let cap_bytes = Self::slot_bytes::<T>(values.capacity())?;
        let len_bytes = Self::slot_bytes::<T>(values.len())?;
        if values.len() < values.capacity() {
            // Shrinking may allocate the exact-size slice while old Vec is live.
            self.transient(len_bytes)?;
        }
        let boxed = values.into_boxed_slice();
        self.release(cap_bytes);
        self.admit(len_bytes)?;
        Ok(boxed)
    }

    pub(in crate::redo_replay) fn release_vec<T>(&mut self, values: Vec<T>) {
        let cap_bytes = Self::slot_bytes::<T>(values.capacity()).unwrap_or(0);
        drop(values);
        self.release(cap_bytes);
    }
}

impl PhysicalRecoveryDecodeStorage for RedoAllocationBudget {
    type Denial = PhysicalRedoPlanningDenial;

    fn admit_allocation(&mut self, bytes: u64) -> Result<(), Self::Denial> {
        self.admit(bytes)
    }
}
