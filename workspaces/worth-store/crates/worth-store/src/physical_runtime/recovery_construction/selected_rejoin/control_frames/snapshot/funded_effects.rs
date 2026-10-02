//! Native backing for exact addressed head-effect media witnesses.

use worth_store_buffer_pool::{OperationAllocationGrant, PhysicalResidencyIncarnation};
use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::store_namespace::StableStoreIdentity;

use super::{SelectedArtifactSlice, SelectedMediaRejoinDenial as Denial};
use crate::physical_runtime::{
    record_serving::RecordBootstrapDenial, LifecycleGeneration, PhysicalRecoveryReadAllocation,
    PhysicalRecoveryRejoinResidentDenial as ResidentDenial, PhysicalScopedAllocationFailure,
};

pub(in crate::physical_runtime::recovery_construction) struct FundedHeadEffectSlices {
    slices: Vec<SelectedArtifactSlice>,
    store: StableStoreIdentity,
    pool: PhysicalResidencyIncarnation,
    origin: LifecycleGeneration,
    // Storage is disposed before its independently owned grant.
    backing: Option<OperationAllocationGrant>,
}

impl FundedHeadEffectSlices {
    pub(in crate::physical_runtime::recovery_construction) fn prepare(
        window: &PhysicalRecoveryReadAllocation<'_>,
        count: usize,
    ) -> Result<Self, Denial> {
        let origin = window
            .recovery_origin_generation()
            .ok_or(Denial::Resident(ResidentDenial::MissingResidentAdmission))?;
        let requested = slot_bytes(count)?;
        let backing = window.reserve_owned(requested).map_err(Denial::Resident)?;
        let mut slices = Vec::new();
        slices
            .try_reserve_exact(count)
            .map_err(|cause| Denial::Resident(ResidentDenial::Allocation { requested, cause }))?;
        let actual = slot_bytes(slices.capacity())?;
        if actual != requested {
            return Err(Denial::Resident(
                ResidentDenial::AllocatorExceededReservation { requested, actual },
            ));
        }
        Ok(Self {
            slices,
            store: window.store_identity(),
            pool: window.pool_identity(),
            origin,
            backing,
        })
    }

    pub(in crate::physical_runtime::recovery_construction) fn push(
        &mut self,
        slice: SelectedArtifactSlice,
    ) -> Result<(), Denial> {
        if self.slices.len() == self.slices.capacity() {
            return Err(Denial::BoundExceeded);
        }
        self.slices.push(slice);
        Ok(())
    }

    pub(in crate::physical_runtime::recovery_construction) fn slices(
        &self,
    ) -> &[SelectedArtifactSlice] {
        &self.slices
    }

    pub(in crate::physical_runtime::recovery_construction) fn owned_heap_bytes(
        &self,
    ) -> Option<u64> {
        slot_bytes(self.slices.capacity()).ok()
    }

    pub(in crate::physical_runtime::recovery_construction) fn charged_bytes(&self) -> u64 {
        self.backing
            .as_ref()
            .map_or(0, OperationAllocationGrant::bytes)
    }

    pub(in crate::physical_runtime::recovery_construction) fn same_bytes(
        &self,
        other: &Self,
    ) -> bool {
        self.slices == other.slices
    }

    pub(in crate::physical_runtime::recovery_construction) fn matching_owner(
        &self,
        window: &PhysicalRecoveryReadAllocation<'_>,
    ) -> bool {
        self.store == window.store_identity()
            && self.pool == window.pool_identity()
            && Some(self.origin) == window.recovery_origin_generation()
    }

    pub(super) fn can_merge(&self, other: &Self) -> bool {
        self.store == other.store && self.pool == other.pool && self.origin == other.origin
    }

    pub(super) fn merged_owned_heap_bytes(&self, other: &Self) -> Result<u64, Denial> {
        slot_bytes(self.merged_capacity(other)?)
    }

    fn merged_capacity(&self, other: &Self) -> Result<usize, Denial> {
        if other.slices.is_empty() {
            return Ok(self.slices.capacity());
        }
        if self.slices.is_empty() {
            return Ok(other.slices.capacity());
        }
        let count = self
            .slices
            .len()
            .checked_add(other.slices.len())
            .ok_or(Denial::BoundExceeded)?;
        if count <= self.slices.capacity() {
            return Ok(self.slices.capacity());
        }
        self.slices
            .capacity()
            .checked_mul(2)
            .map(|capacity| capacity.max(count))
            .ok_or(Denial::BoundExceeded)
    }

    /// The destination's old and fresh vectors overlap while the donor's
    /// separate grant remains live. A failed preparation leaves both data and
    /// destination charge unchanged; no read-window or new pool is minted.
    pub(super) fn merge(&mut self, mut other: Self) -> Result<(), Denial> {
        if !self.can_merge(&other) {
            return Err(Denial::RootBinding);
        }
        if other.slices.is_empty() {
            return Ok(());
        }
        if self.slices.is_empty() {
            std::mem::swap(self, &mut other);
            return Ok(());
        }
        let capacity = self.merged_capacity(&other)?;
        if capacity == self.slices.capacity() {
            self.slices.append(&mut other.slices);
            return Ok(());
        }
        // Geometric retained backing bounds prefix copies across an ordered
        // replay; spare slots stay charged and subsequent donors use them.
        let requested = slot_bytes(capacity)?;
        let old_charge = self.charged_bytes();
        let peak = old_charge
            .checked_add(requested)
            .ok_or(Denial::BoundExceeded)?;
        self.resize_charge(peak)?;
        let mut fresh = Vec::new();
        if let Err(cause) = fresh.try_reserve_exact(capacity) {
            self.resize_charge(old_charge)?;
            return Err(Denial::Resident(ResidentDenial::Allocation {
                requested,
                cause,
            }));
        }
        let actual = slot_bytes(fresh.capacity())?;
        if actual != requested {
            drop(fresh);
            self.resize_charge(old_charge)?;
            return Err(Denial::Resident(
                ResidentDenial::AllocatorExceededReservation { requested, actual },
            ));
        }
        fresh.extend_from_slice(&self.slices);
        fresh.extend_from_slice(&other.slices);
        let old = std::mem::replace(&mut self.slices, fresh);
        drop(old);
        drop(other);
        self.resize_charge(actual)
    }

    fn resize_charge(&mut self, bytes: u64) -> Result<(), Denial> {
        self.backing
            .as_mut()
            .ok_or(Denial::BoundExceeded)?
            .try_resize(bytes)
            .map_err(|cause| {
                Denial::Resident(ResidentDenial::OperationAllocation(
                    PhysicalScopedAllocationFailure::from_denial(cause, self.origin),
                ))
            })
    }

    pub(super) fn verify_serving_media(
        &self,
        media: &QualifiedFilesystemMedia,
        window: &mut PhysicalRecoveryReadAllocation<'_>,
    ) -> Result<bool, RecordBootstrapDenial> {
        if !self.matching_owner(window) {
            return Err(RecordBootstrapDenial::RecoveredHeadWitnessOwnerMismatch);
        }
        let bytes = self
            .slices
            .iter()
            .try_fold(0_u64, |sum, slice| sum.checked_add(u64::from(slice.length)))
            .ok_or(RecordBootstrapDenial::RecoveredCheckpointCustodyMismatch)?;
        if self.slices.is_empty() {
            return Ok(true);
        }
        let mut observation = media
            .bounded_record_observation(self.slices.len() as u64, bytes)
            .map_err(RecordBootstrapDenial::RecoveredHeadObservationUnavailable)?;
        for slice in &self.slices {
            if !slice.matches_funded_serving_media(&mut observation, window)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn slot_bytes(count: usize) -> Result<u64, Denial> {
    count
        .checked_mul(std::mem::size_of::<SelectedArtifactSlice>())
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .and_then(|bytes| u64::try_from(bytes).ok())
        .ok_or(Denial::BoundExceeded)
}

#[cfg(test)]
mod tests;
