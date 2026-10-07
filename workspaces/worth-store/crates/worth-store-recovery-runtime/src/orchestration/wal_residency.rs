//! Native ownership of Runtime WAL vector slots; element-owned heaps are separate.

use std::{collections::TryReserveError, num::NonZeroU64};
use worth_store::physical_runtime::{
    PhysicalRecoveryCoordination, PhysicalRecoveryRejoinResidentDenial,
    PhysicalRecoveryWalInventoryBacking, RecoveryWalAllocationDenial as Denial,
};

#[derive(Debug)]
pub(crate) struct NativeWalRoster<T> {
    // Declaration order keeps the reservation alive throughout vector disposal.
    values: Vec<T>,
    extra_shared_bytes: u64,
    allocation_poison: Option<Denial>,
    backing: Option<PhysicalRecoveryWalInventoryBacking>,
}

impl<T> NativeWalRoster<T> {
    pub(crate) const fn empty(extra_shared_bytes: u64) -> Self {
        Self {
            values: Vec::new(),
            extra_shared_bytes,
            allocation_poison: None,
            backing: None,
        }
    }

    pub(crate) fn with_capacity(
        owner: &PhysicalRecoveryCoordination,
        count: usize,
    ) -> Result<Self, Denial> {
        let mut roster = Self::empty(0);
        if count != 0 {
            roster.grow_capacity(owner, count)?;
        }
        Ok(roster)
    }

    pub(crate) fn reserve_one(
        &mut self,
        owner: &PhysicalRecoveryCoordination,
    ) -> Result<(), Denial> {
        if let Some(cause) = &self.allocation_poison {
            return Err(cause.clone());
        }
        if self.len() < self.capacity() {
            return Ok(());
        }
        let capacity = self
            .capacity()
            .checked_mul(2)
            .ok_or(Denial::SizeOverflow)?
            .max(1);
        self.grow_capacity(owner, capacity)
    }

    pub(crate) fn push_reserved(&mut self, value: T) {
        assert!(
            self.allocation_poison.is_none(),
            "poisoned WAL allocation cannot append"
        );
        assert!(
            self.len() < self.capacity(),
            "WAL slot must be reserved before push"
        );
        self.values.push(value);
    }

    pub(crate) fn len(&self) -> usize {
        self.values.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub(crate) fn capacity(&self) -> usize {
        self.values.capacity()
    }

    pub(crate) fn as_slice(&self) -> &[T] {
        &self.values
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.values
    }

    pub(crate) fn charged_bytes(&self) -> u64 {
        self.backing
            .as_ref()
            .map_or(0, |backing| backing.charged_bytes())
    }

    pub(crate) fn owned_heap_bytes(&self) -> Option<u64> {
        if self.capacity() == 0 {
            return Some(0);
        }
        self.extra_shared_bytes
            .checked_add(vector_bytes::<T>(self.capacity()).ok()?)
    }

    fn grow_capacity(
        &mut self,
        owner: &PhysicalRecoveryCoordination,
        capacity: usize,
    ) -> Result<(), Denial> {
        let old_bytes = vector_bytes::<T>(self.capacity())?;
        let new_bytes = vector_bytes::<T>(capacity)?;
        let peak = self
            .extra_shared_bytes
            .checked_add(old_bytes)
            .and_then(|bytes| bytes.checked_add(new_bytes))
            .ok_or(Denial::SizeOverflow)?;
        self.reserve_peak(owner, peak)?;
        let mut replacement = Vec::new();
        if let Err(cause) = replacement.try_reserve_exact(capacity) {
            drop(replacement);
            self.restore_after_disposal(old_bytes)?;
            return Err(allocator_denial(peak, cause));
        }
        if replacement.capacity() != capacity {
            let actual = self
                .extra_shared_bytes
                .checked_add(old_bytes)
                .and_then(|bytes| {
                    vector_bytes::<T>(replacement.capacity())
                        .ok()
                        .and_then(|new| bytes.checked_add(new))
                })
                .unwrap_or(u64::MAX);
            drop(replacement);
            let cause = Denial::AllocatorExceededReservation {
                requested: peak,
                actual,
            };
            self.allocation_poison = Some(cause.clone());
            self.restore_after_disposal(old_bytes)?;
            return Err(cause);
        }
        replacement.append(&mut self.values);
        let old = std::mem::replace(&mut self.values, replacement);
        drop(old);
        self.restore_after_disposal(new_bytes)
    }

    fn reserve_peak(
        &mut self,
        owner: &PhysicalRecoveryCoordination,
        required: u64,
    ) -> Result<(), Denial> {
        match &mut self.backing {
            Some(backing) => backing.grow_total(required),
            None => {
                let bytes = NonZeroU64::new(required).ok_or(Denial::SizeOverflow)?;
                self.backing = Some(owner.admit_wal_inventory_backing(bytes)?);
                Ok(())
            }
        }
    }

    fn restore_after_disposal(&mut self, vector_bytes: u64) -> Result<(), Denial> {
        if vector_bytes == 0 {
            drop(self.backing.take());
            return Ok(());
        }
        let retained = self
            .extra_shared_bytes
            .checked_add(vector_bytes)
            .ok_or(Denial::SizeOverflow)?;
        let result = self
            .backing
            .as_mut()
            .expect("nonempty WAL capacity has backing")
            .settle_after_disposal(retained);
        if let Err(cause) = &result {
            self.allocation_poison = Some(cause.clone());
        }
        result
    }
}

fn vector_bytes<T>(capacity: usize) -> Result<u64, Denial> {
    if std::mem::size_of::<T>() == 0 {
        return Err(Denial::SizeOverflow);
    }
    capacity
        .checked_mul(std::mem::size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::SizeOverflow)
}

fn allocator_denial(requested: u64, cause: TryReserveError) -> Denial {
    Denial::Backing {
        requested,
        cause: PhysicalRecoveryRejoinResidentDenial::Allocation { requested, cause },
    }
}

#[cfg(all(test, feature = "certification-test-authority"))]
mod tests;
