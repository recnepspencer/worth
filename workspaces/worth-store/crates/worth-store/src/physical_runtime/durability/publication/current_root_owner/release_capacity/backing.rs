//! Mandatory publication backing funded by the Store's real Recovery pool.
//! Re-census retained capacities, including spare backing after cancellation.

mod live_allocation;
mod window;
pub(in crate::physical_runtime) use live_allocation::ReleasePublicationAllocationOwner;
pub(in crate::physical_runtime::durability::publication::current_root_owner) use live_allocation::{
    FundedRootFrame, LiveReleaseAllocation,
};
pub(super) use live_allocation::SHARED_CUSTODY_BYTES;
pub(super) use window::LiveBackingWindow;

use super::{ReleaseCertificateCapacityDenial as Denial, SelectedReleaseCustodyLedger};
use crate::physical_runtime::{
    PhysicalRecoveryAllocationAdmission, PhysicalRecoveryRejoinResidentDenial,
};
use std::sync::Arc;
use worth_store_physical_format::{DurablePhysicalRootManifest, ReleaseCustodyHeadKeyV1};

const PENDING_CONTROL_BYTES: u64 =
    std::mem::size_of::<super::super::certificate_capacity::CheckpointCustodyState>() as u64;

impl SelectedReleaseCustodyLedger {
    /// The committed checkpoint roster is not here: the standing checkpoint
    /// reservation holds it, so its bytes are charged exactly once.
    pub(super) fn publication_backing_bytes(&self) -> Option<u64> {
        self.effective_heads
            .owned_heap_bytes()?
            .checked_add(vector_heap_bytes(&self.pending_events)?)
    }

    pub(super) fn retained_requirement(
        &self,
        ceiling: PhysicalRecoveryAllocationAdmission,
        closure: u64,
        fence: u64,
    ) -> Result<u64, Denial> {
        self.publication_backing_bytes()
            .and_then(|bytes| bytes.checked_add(closure))
            .and_then(|bytes| bytes.checked_add(fence))
            .and_then(|bytes| bytes.checked_add(PENDING_CONTROL_BYTES))
            .and_then(|bytes| bytes.checked_add(SHARED_CUSTODY_BYTES))
            .ok_or(Denial::Resident(
                PhysicalRecoveryRejoinResidentDenial::SizeOverflow {
                    admitted: ceiling.byte_limit(),
                },
            ))
    }

    pub(super) fn prepare_control_backing(
        &mut self,
        owner: &ReleasePublicationAllocationOwner,
        ceiling: PhysicalRecoveryAllocationAdmission,
        fence_bytes: u64,
    ) -> Result<(), Denial> {
        let required = self.retained_requirement(ceiling, 0, fence_bytes)?;
        owner.fund(&mut self.allocation_custody, ceiling, required)
    }

    /// One drop's admission: the publication backing, then the standing
    /// checkpoint reservation grown to the envelope that holds this drop's
    /// Batch. Either denial happens before any effect, so a drop is never
    /// admitted unless its checkpoint is already funded.
    pub(super) fn admit_drop_backing(
        &mut self,
        owner: &ReleasePublicationAllocationOwner,
        ceiling: PhysicalRecoveryAllocationAdmission,
        closure_bytes: u64,
        fence_bytes: u64,
        key: ReleaseCustodyHeadKeyV1,
    ) -> Result<FundedRootFrame, Denial> {
        let root_frame =
            self.prepare_publication_backing(owner, ceiling, closure_bytes, fence_bytes, key)?;
        self.reserve_capture_custody(owner, ceiling, Some(key))?;
        Ok(root_frame)
    }

    fn prepare_publication_backing(
        &mut self,
        owner: &ReleasePublicationAllocationOwner,
        ceiling: PhysicalRecoveryAllocationAdmission,
        closure_bytes: u64,
        fence_bytes: u64,
        key: ReleaseCustodyHeadKeyV1,
    ) -> Result<FundedRootFrame, Denial> {
        let already_live = self.retained_requirement(ceiling, closure_bytes, fence_bytes)?;
        let mut window =
            LiveBackingWindow::new(owner, &mut self.allocation_custody, ceiling, already_live)?;
        window.grow_vec(&mut self.pending_events, 1)?;
        self.effective_heads
            .reserve_for_key_live(key, &mut window)?;
        let bytes =
            window.reserve_vec(DurablePhysicalRootManifest::maximum_encoding_scratch_bytes())?;
        drop(window);
        Ok(FundedRootFrame::new(
            bytes,
            Arc::clone(
                self.allocation_custody
                    .as_ref()
                    .expect("live funding precedes every allocation"),
            ),
        ))
    }
}

pub(in super::super) fn vector_heap_bytes<T>(values: &Vec<T>) -> Option<u64> {
    values
        .capacity()
        .checked_mul(std::mem::size_of::<T>())?
        .try_into()
        .ok()
}

#[cfg(test)]
#[path = "backing/tests.rs"]
mod tests;
