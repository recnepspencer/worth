use std::{collections::VecDeque, num::NonZeroUsize, sync::Arc};

use super::candidate_allocation::{CandidateFrameAllocator, ProcessCandidateFrameAllocator};
use crate::physical_residency::{
    operation_allocation::OperationAllocationUse, pool::PoolInner, DirtyPhysicalFrame,
    PhysicalCandidateFrameKey, PhysicalFrameKey, PhysicalResidencyDenial,
};

#[derive(Debug)]
pub struct PhysicalCandidateBatchAdmission<'grant> {
    pub(crate) owner: Arc<PoolInner>,
    pub(crate) allocation_use: OperationAllocationUse<'grant>,
    pub(crate) candidate_count: NonZeroUsize,
}

#[derive(Debug)]
pub struct PhysicalCandidateBatchReservation<'grant> {
    pub(crate) owner: Arc<PoolInner>,
    pub(crate) keys: VecDeque<PhysicalCandidateFrameKey>,
    pub(crate) allocation_use: OperationAllocationUse<'grant>,
    pub(crate) armed: bool,
}

#[derive(Debug)]
pub struct PhysicalCandidateFrameReservation {
    pub(crate) owner: Arc<PoolInner>,
    pub(crate) candidate: PhysicalCandidateFrameKey,
    pub(crate) scope: crate::PhysicalOperationAllocationScope,
    pub(crate) armed: bool,
}

impl<'grant> PhysicalCandidateBatchAdmission<'grant> {
    pub fn reserve(
        self,
        keys: &[PhysicalCandidateFrameKey],
    ) -> Result<PhysicalCandidateBatchReservation<'grant>, PhysicalResidencyDenial> {
        let owner = Arc::clone(&self.owner);
        owner.admit_candidate_batch(self, keys)
    }

    pub(crate) fn scope(&self) -> crate::PhysicalOperationAllocationScope {
        self.allocation_use.scope()
    }
}

impl PhysicalCandidateBatchReservation<'_> {
    pub fn reserve_next(
        &mut self,
        candidate: PhysicalCandidateFrameKey,
    ) -> Result<PhysicalCandidateFrameReservation, PhysicalResidencyDenial> {
        let scope = self.allocation_use.scope();
        if self.keys.front().copied() != Some(candidate) {
            return Err(self
                .owner
                .record_denial(PhysicalResidencyDenial::CandidateSequenceConflict));
        }
        self.owner.reserve_next_candidate(scope, candidate)?;
        self.keys.pop_front();
        Ok(PhysicalCandidateFrameReservation::new(
            Arc::clone(&self.owner),
            candidate,
            scope,
        ))
    }
}

impl Drop for PhysicalCandidateBatchReservation<'_> {
    fn drop(&mut self) {
        if self.armed {
            self.owner.finish_candidate_batch();
            self.armed = false;
        }
    }
}

impl PhysicalCandidateFrameReservation {
    pub const fn key(&self) -> PhysicalFrameKey {
        self.candidate.frame_key()
    }

    pub fn materialize<F>(self, fill: F) -> Result<DirtyPhysicalFrame, PhysicalResidencyDenial>
    where
        F: FnOnce(&mut [u8]),
    {
        self.materialize_with_allocator(&ProcessCandidateFrameAllocator, fill)
    }

    pub(crate) fn materialize_with_allocator<F>(
        mut self,
        allocator: &dyn CandidateFrameAllocator,
        fill: F,
    ) -> Result<DirtyPhysicalFrame, PhysicalResidencyDenial>
    where
        F: FnOnce(&mut [u8]),
    {
        let mut fill = Some(fill);
        self.materialize_filled_candidate(allocator, &mut |bytes| {
            fill.take().expect("candidate fill runs once")(bytes);
        })
    }

    /// One compiled candidate materialization for every fill closure.
    #[inline(never)]
    fn materialize_filled_candidate(
        &mut self,
        allocator: &dyn CandidateFrameAllocator,
        fill: &mut dyn FnMut(&mut [u8]),
    ) -> Result<DirtyPhysicalFrame, PhysicalResidencyDenial> {
        let key = self.candidate.frame_key();
        let length = usize::try_from(key.coordinate().length())
            .expect("physical frame lengths are admitted from u32 coordinates");
        let mut buffer = allocator.allocate(length).map_err(|()| {
            self.owner.candidate_allocator_failed(key);
            self.armed = false;
            PhysicalResidencyDenial::AllocationFailed
        })?;
        let actual = u64::try_from(buffer.capacity()).expect("Vec capacity fits u64");
        let requested = u64::try_from(length).expect("frame length fits u64");
        if actual > requested {
            self.owner.candidate_allocator_failed(key);
            self.armed = false;
            return Err(PhysicalResidencyDenial::AllocatorExceededReservation {
                requested,
                actual,
            });
        }
        self.owner.actualize_allocation(
            crate::physical_residency::PhysicalResidencyAllocationActualization::new(
                crate::PhysicalResidencyDimension::ResidentBytes,
                self.scope,
                crate::physical_residency::PhysicalResidencyRequestedAllocationUnits::new(
                    requested,
                ),
                crate::physical_residency::PhysicalResidencyActualAllocationUnits::new(actual),
            ),
        );
        fill(buffer.as_mut_slice());
        let bytes = buffer.into_resident();
        let frame = self.owner.finish_candidate(key, bytes)?;
        self.armed = false;
        Ok(frame)
    }
}

impl Drop for PhysicalCandidateFrameReservation {
    fn drop(&mut self) {
        if self.armed {
            self.owner.cancel_candidate(self.candidate.frame_key());
        }
    }
}
