//! The standing checkpoint reservation. One grant holds the committed
//! checkpoint roster and the next capture's whole envelope, so a drop that
//! admission accepts can always be checkpointed without fresh funding.
//!
//! Sizing: `max(roster capacity, envelope heads) + envelope`. While a capture
//! is in flight, the old roster and the capture's buffers are both live, and
//! the first term covers the old roster. After the commit, the new roster
//! (the envelope's fold heads) replaces the old one. Between commits, heads
//! and Batches only grow; a commit only shrinks them. So the requirement
//! after the commit never exceeds the requirement before it, and the same
//! grant is re-established for the remaining state without a new grant.
use std::sync::Arc;

use worth_store_physical_format::{ReleaseCustodyHeadEntryV1, ReleaseCustodyHeadKeyV1};

use super::super::backing::{LiveReleaseAllocation, ReleasePublicationAllocationOwner};
use super::super::{ReleaseCertificateCapacityDenial as Denial, SelectedReleaseCustodyLedger};
use super::CheckpointPreparation;
use crate::physical_runtime::PhysicalRecoveryAllocationAdmission;

impl SelectedReleaseCustodyLedger {
    /// Bytes the standing reservation must hold for the current state plus
    /// an optional incoming drop. A tier slot is always held, so a later
    /// tier activation never needs to grow it.
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn capture_custody_requirement(
        &self,
        incoming: Option<ReleaseCustodyHeadKeyV1>,
    ) -> Result<u64, Denial> {
        let envelope = self.capture_envelope_bytes(incoming, true)?;
        let heads = u64::try_from(envelope.heads)
            .ok()
            .and_then(|heads| {
                heads.checked_mul(std::mem::size_of::<ReleaseCustodyHeadEntryV1>() as u64)
            })
            .ok_or(Denial::CapacityExhausted)?;
        let roster = self
            .checkpoint_heads
            .owned_heap_bytes()
            .ok_or(Denial::CapacityExhausted)?;
        roster
            .max(heads)
            .checked_add(envelope.bytes)
            .ok_or(Denial::CapacityExhausted)
    }

    /// Grows the standing reservation before any effect. A pool that cannot
    /// grow it is a typed denial; the reservation never shrinks.
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn reserve_capture_custody(
        &mut self,
        owner: &ReleasePublicationAllocationOwner,
        ceiling: PhysicalRecoveryAllocationAdmission,
        incoming: Option<ReleaseCustodyHeadKeyV1>,
    ) -> Result<(), Denial> {
        let required = self.capture_custody_requirement(incoming)?;
        owner.fund(&mut self.capture_custody, ceiling, required)
    }

    /// A capture consumes the standing reservation. Only a capture that
    /// overlaps a still-live earlier capture funds its own envelope.
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn prepare_capture(
        &mut self,
        owner: &ReleasePublicationAllocationOwner,
        ceiling: PhysicalRecoveryAllocationAdmission,
        tier: bool,
    ) -> Result<CheckpointPreparation, Denial> {
        let envelope = self.checkpoint_capture_envelope(None, tier)?;
        // Clones happen only under the publication lock, so a count of one
        // cannot rise underneath this check; a stale higher count is safe.
        let custody = if self.capture_custody.as_ref().map_or(1, Arc::strong_count) == 1 {
            self.reserve_capture_custody(owner, ceiling, None)?;
            self.capture_custody.clone()
        } else {
            let mut overlapping: Option<Arc<LiveReleaseAllocation>> = None;
            owner.fund(&mut overlapping, ceiling, envelope.bytes())?;
            overlapping
        };
        Ok(envelope.prepare(custody.ok_or(Denial::CapacityExhausted)?))
    }

    #[cfg(test)]
    pub(in crate::physical_runtime::durability::publication::current_root_owner) fn capture_custody_bytes(
        &self,
    ) -> Option<u64> {
        self.capture_custody.as_ref().map(|custody| custody.bytes())
    }
}
