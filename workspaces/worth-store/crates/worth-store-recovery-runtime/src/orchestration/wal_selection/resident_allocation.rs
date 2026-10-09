//! One retained native reservation for candidate facts and tail partition storage.

use std::num::NonZeroU64;
use worth_store::physical_runtime::{
    PhysicalRecoveryCoordination, PhysicalRecoveryRejoinResidentDenial,
    PhysicalRecoveryWalInventoryBacking, RecoveryWalAllocationDenial as Denial,
};

#[derive(Debug, Default)]
pub(super) struct WalSelectionAllocation {
    backing: Option<PhysicalRecoveryWalInventoryBacking>,
    poison: Option<Denial>,
}

impl WalSelectionAllocation {
    pub(super) fn charged_bytes(&self) -> u64 {
        self.backing
            .as_ref()
            .map_or(0, |backing| backing.charged_bytes())
    }

    /// Retained storage remains live while the prospective vector is prepared.
    pub(super) fn prepare_vector<T>(
        &mut self,
        owner: &PhysicalRecoveryCoordination,
        count: usize,
    ) -> Result<Vec<T>, Denial> {
        if let Some(cause) = &self.poison {
            return Err(cause.clone());
        }
        if count == 0 {
            return Ok(Vec::new());
        }
        let before = self.charged_bytes();
        let required = before
            .checked_add(vector_bytes::<T>(count)?)
            .ok_or(Denial::SizeOverflow)?;
        match &mut self.backing {
            Some(backing) => backing.grow_total(required)?,
            None => {
                self.backing = Some(owner.admit_wal_inventory_backing(
                    NonZeroU64::new(required).ok_or(Denial::SizeOverflow)?,
                )?)
            }
        }
        let mut values = Vec::new();
        if let Err(cause) = values.try_reserve_exact(count) {
            drop(values);
            self.settle_after_disposal(before)?;
            return Err(Denial::Backing {
                requested: required,
                cause: PhysicalRecoveryRejoinResidentDenial::Allocation {
                    requested: required,
                    cause,
                },
            });
        }
        if values.capacity() != count {
            let actual = vector_bytes::<T>(values.capacity())
                .ok()
                .and_then(|bytes| before.checked_add(bytes))
                .unwrap_or(u64::MAX);
            drop(values);
            let cause = Denial::AllocatorExceededReservation {
                requested: required,
                actual,
            };
            self.poison = Some(cause.clone());
            self.settle_after_disposal(before)?;
            return Err(cause);
        }
        Ok(values)
    }

    pub(super) fn settle_after_disposal(&mut self, retained: u64) -> Result<(), Denial> {
        match &mut self.backing {
            Some(backing) => backing.settle_after_disposal(retained)?,
            None if retained == 0 => return Ok(()),
            None => {
                return Err(Denial::AllocatorExceededReservation {
                    requested: 0,
                    actual: retained,
                })
            }
        }
        if retained == 0 {
            drop(self.backing.take());
        }
        Ok(())
    }
}

pub(super) fn vector_bytes<T>(capacity: usize) -> Result<u64, Denial> {
    capacity
        .checked_mul(std::mem::size_of::<T>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::SizeOverflow)
}
