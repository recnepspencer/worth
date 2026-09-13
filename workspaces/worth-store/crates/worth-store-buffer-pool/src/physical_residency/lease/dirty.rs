use super::dirty_replacement_allocation::{
    DirtyReplacementAllocator, ProcessDirtyReplacementAllocator,
};
use super::PhysicalFrameLease;
use crate::physical_residency::pool::PoolInner;
use crate::physical_residency::pool_ownership::CandidateFrameCleanAuthority;
use crate::{PhysicalResidencyDenial, PhysicalResidencyIncarnation};
use std::sync::Arc;

#[derive(Debug)]
pub struct DirtyPhysicalFrame {
    pub(crate) lease: Option<PhysicalFrameLease>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PhysicalDirtyReplacementError<E> {
    Residency(PhysicalResidencyDenial),
    Fill(E),
}

#[derive(Debug)]
pub struct PhysicalDirtyReplacementReservation<'grant> {
    owner: Arc<PoolInner>,
    lease: Option<PhysicalFrameLease>,
    allocation_use: super::super::operation_allocation::OperationAllocationUse<'grant>,
    bytes: u64,
    armed: bool,
}

impl<'grant> PhysicalDirtyReplacementReservation<'grant> {
    pub(super) fn new(
        lease: PhysicalFrameLease,
        allocation_use: super::super::operation_allocation::OperationAllocationUse<'grant>,
        bytes: u64,
    ) -> Self {
        Self {
            owner: Arc::clone(&lease.owner),
            lease: Some(lease),
            allocation_use,
            bytes,
            armed: true,
        }
    }

    pub fn replace<E, F>(
        self,
        fill: F,
    ) -> Result<DirtyPhysicalFrame, PhysicalDirtyReplacementError<E>>
    where
        F: FnOnce(&[u8], &mut [u8]) -> Result<(), E>,
    {
        self.replace_with_allocator(&ProcessDirtyReplacementAllocator, fill)
    }

    pub(crate) fn replace_with_allocator<E, F>(
        mut self,
        allocator: &dyn DirtyReplacementAllocator,
        fill: F,
    ) -> Result<DirtyPhysicalFrame, PhysicalDirtyReplacementError<E>>
    where
        F: FnOnce(&[u8], &mut [u8]) -> Result<(), E>,
    {
        let length = usize::try_from(self.bytes)
            .expect("physical frame lengths are admitted from u32 coordinates");
        let mut replacement = allocator.allocate(length).map_err(|()| {
            self.release_after_allocator_failure();
            PhysicalDirtyReplacementError::Residency(PhysicalResidencyDenial::AllocationFailed)
        })?;
        let actual = u64::try_from(replacement.capacity()).expect("Vec capacity fits u64");
        if actual > self.bytes {
            let requested = self.bytes;
            self.release_after_allocator_failure();
            return Err(PhysicalDirtyReplacementError::Residency(
                PhysicalResidencyDenial::AllocatorExceededReservation { requested, actual },
            ));
        }
        self.owner.actualize_allocation(
            crate::physical_residency::PhysicalResidencyAllocationActualization::new(
                crate::PhysicalResidencyDimension::DirtyReplacementBytes,
                self.allocation_use.scope(),
                crate::physical_residency::PhysicalResidencyRequestedAllocationUnits::new(
                    self.bytes,
                ),
                crate::physical_residency::PhysicalResidencyActualAllocationUnits::new(actual),
            ),
        );
        let lease = self
            .lease
            .as_ref()
            .expect("armed dirty replacement retains its clean lease");
        if let Err(error) = fill(lease.bytes.as_slice(), replacement.as_mut_slice()) {
            self.release_reservation();
            return Err(PhysicalDirtyReplacementError::Fill(error));
        }
        let replacement = replacement.into_resident();
        let mut lease = self
            .lease
            .take()
            .expect("armed dirty replacement retains its clean lease");
        let resident_generation = match lease.owner.finish_dirty_replacement(
            self.allocation_use.scope(),
            lease.key,
            &lease.bytes,
            Arc::clone(&replacement),
        ) {
            Ok(generation) => generation,
            Err(reason) => {
                self.release_reservation();
                return Err(PhysicalDirtyReplacementError::Residency(reason));
            }
        };
        lease.bytes = replacement;
        lease.resident_generation = resident_generation;
        self.release_reservation();
        Ok(DirtyPhysicalFrame { lease: Some(lease) })
    }

    fn release_reservation(&mut self) {
        if self.armed {
            self.owner
                .release_dirty_replacement(self.allocation_use.scope(), self.bytes);
            self.armed = false;
        }
    }

    fn release_after_allocator_failure(&mut self) {
        if self.armed {
            self.owner
                .dirty_replacement_allocator_failed(self.allocation_use.scope(), self.bytes);
            self.armed = false;
        }
    }
}

impl Drop for PhysicalDirtyReplacementReservation<'_> {
    fn drop(&mut self) {
        self.release_reservation();
    }
}

impl DirtyPhysicalFrame {
    pub fn resident_generation(
        &self,
    ) -> crate::physical_residency::PhysicalResidentFrameGeneration {
        self.lease
            .as_ref()
            .expect("dirty physical frame retains its lease")
            .resident_generation()
    }

    pub fn pool_incarnation(&self) -> PhysicalResidencyIncarnation {
        self.lease
            .as_ref()
            .expect("dirty frame lease is present")
            .owner
            .incarnation()
    }

    pub fn bytes(&self) -> &[u8] {
        self.lease
            .as_ref()
            .expect("dirty frame lease is present")
            .bytes
            .as_slice()
    }

    pub fn complete_candidate_publication(
        mut self,
        authority: &CandidateFrameCleanAuthority,
    ) -> Result<PhysicalFrameLease, PhysicalResidencyDenial> {
        let lease = self.lease.as_ref().expect("dirty frame lease is present");
        if !authority.authorizes(&lease.owner) {
            return Err(PhysicalResidencyDenial::CandidateCleanAuthorityMismatch);
        }
        let lease = self.lease.take().expect("dirty frame lease is present");
        lease.owner.publish_clean(lease.key)?;
        Ok(lease)
    }

    pub fn discard_candidate(mut self) -> Result<(), PhysicalResidencyDenial> {
        let lease = self.lease.take().expect("dirty frame lease is present");
        lease.owner.discard_dirty_candidate(lease.key)
    }
}
