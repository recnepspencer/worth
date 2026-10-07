//! Native ownership of inline inventory slots, not their already-funded C9 heaps.
use crate::physical_runtime::{
    PhysicalRecoveryCoordination, PhysicalRecoveryRejoinResidentDenial,
    PhysicalRecoveryWalInventoryBacking, RecoveryWalAllocationDenial as Denial,
};
use std::{num::NonZeroU64, ops::Deref};

pub(super) struct WalRoster<T> {
    pub(super) values: Vec<T>,
    failed: Option<Denial>,
    backing: Option<PhysicalRecoveryWalInventoryBacking>,
}
impl<T> WalRoster<T> {
    pub(super) fn empty() -> Self {
        Self {
            values: Vec::new(),
            failed: None,
            backing: None,
        }
    }
    pub(super) fn with_capacity(
        owner: &PhysicalRecoveryCoordination,
        count: usize,
    ) -> Result<Self, Denial> {
        let mut roster = Self::empty();
        if count != 0 {
            roster.prepare(owner, count)?;
        }
        Ok(roster)
    }
    pub(super) fn capacity(&self) -> usize {
        self.values.capacity()
    }
    pub(super) fn owned_heap_bytes(&self) -> Option<u64> {
        bytes::<T>(self.capacity()).ok()
    }
    #[cfg(test)]
    pub(super) fn charged_bytes(&self) -> u64 {
        self.backing
            .as_ref()
            .map_or(0, PhysicalRecoveryWalInventoryBacking::charged_bytes)
    }
    pub(super) fn next_capacity(&self) -> Result<usize, Denial> {
        self.capacity()
            .checked_mul(2)
            .map(|count| count.max(1))
            .ok_or(Denial::SizeOverflow)
    }
    pub(super) fn reserve_one(
        &mut self,
        owner: &PhysicalRecoveryCoordination,
    ) -> Result<(), Denial> {
        if let Some(cause) = &self.failed {
            return Err(cause.clone());
        }
        if self.len() < self.capacity() {
            return Ok(());
        }
        self.prepare(owner, self.next_capacity()?)
    }
    fn prepare(
        &mut self,
        owner: &PhysicalRecoveryCoordination,
        count: usize,
    ) -> Result<(), Denial> {
        let retained = bytes::<T>(self.capacity())?;
        let prospective = bytes::<T>(count)?;
        let simultaneous = retained
            .checked_add(prospective)
            .ok_or(Denial::SizeOverflow)?;
        match &mut self.backing {
            Some(backing) => backing.grow_total(simultaneous)?,
            None => {
                self.backing = Some(owner.admit_wal_inventory_backing(
                    NonZeroU64::new(simultaneous).ok_or(Denial::SizeOverflow)?,
                )?)
            }
        }
        let mut prepared = Vec::new();
        if let Err(cause) = prepared.try_reserve_exact(count) {
            drop(prepared);
            self.settle(retained)?;
            return Err(Denial::Backing {
                requested: prospective,
                cause: PhysicalRecoveryRejoinResidentDenial::Allocation {
                    requested: prospective,
                    cause,
                },
            });
        }
        let actual = bytes::<T>(prepared.capacity())?;
        if actual != prospective {
            drop(prepared);
            self.settle(retained)?;
            let cause = Denial::AllocatorExceededReservation {
                requested: prospective,
                actual,
            };
            self.failed = Some(cause.clone());
            return Err(cause);
        }
        let mut old = std::mem::take(&mut self.values);
        prepared.append(&mut old);
        self.values = prepared;
        drop(old);
        self.settle(prospective)
    }
    fn settle(&mut self, retained: u64) -> Result<(), Denial> {
        if let Some(backing) = &mut self.backing {
            backing.settle_after_disposal(retained)?;
        }
        if retained == 0 {
            self.backing = None;
        }
        Ok(())
    }
    pub(super) fn push_reserved(&mut self, value: T) {
        assert!(
            self.failed.is_none() && self.len() < self.capacity(),
            "WAL slots must be admitted before append"
        );
        self.values.push(value);
    }
    pub(super) fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.values
    }
}
impl<T> Deref for WalRoster<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}
impl<T: PartialEq> PartialEq for WalRoster<T> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}
impl<T: Eq> Eq for WalRoster<T> {}
pub(super) fn bytes<T>(count: usize) -> Result<u64, Denial> {
    count
        .checked_mul(std::mem::size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::SizeOverflow)
}
