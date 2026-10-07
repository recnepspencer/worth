//! Heap backing owned by a pending claim, excluding shared Arc backing.

use super::{
    PendingReleaseCheckpointBase, VerifiedHistoricalPendingWalBatch,
    VerifiedOrderedPendingWalReleaseBatch, VerifiedPendingWalReleaseCustody,
};

fn boxed_bytes<T>(length: usize) -> Option<u64> {
    u64::try_from(length)
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?)
}

impl VerifiedPendingWalReleaseCustody {
    /// The checkpoint and ordered-history Arc backing are counted once by
    /// the enclosing live-owner ledger, not once per claim reference.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let base = match &self.base {
            PendingReleaseCheckpointBase::NoRelease(_) => 0,
            PendingReleaseCheckpointBase::Released(base) => {
                u64::try_from(std::mem::size_of_val(base.as_ref()))
                    .ok()?
                    .checked_add(base.owned_heap_bytes()?)?
            }
            PendingReleaseCheckpointBase::ReleasedAddressed(base) => {
                u64::try_from(std::mem::size_of_val(base.as_ref()))
                    .ok()?
                    .checked_add(base.owned_heap_bytes()?)?
            }
            PendingReleaseCheckpointBase::ReleasedHeadV2(base) => {
                u64::try_from(std::mem::size_of_val(base.as_ref()))
                    .ok()?
                    .checked_add(base.owned_heap_bytes()?)?
            }
        };
        let historical = self.historical_batches.iter().try_fold(
            boxed_bytes::<VerifiedHistoricalPendingWalBatch>(self.historical_batches.len())?,
            |sum, batch| sum.checked_add(batch.owned_heap_bytes()?),
        )?;
        let ordered = self.ordered_released_batches.iter().try_fold(
            boxed_bytes::<VerifiedOrderedPendingWalReleaseBatch>(
                self.ordered_released_batches.len(),
            )?,
            |sum, batch| sum.checked_add(batch.owned_heap_bytes()?),
        )?;
        base.checked_add(historical)?
            .checked_add(ordered)?
            .checked_add(
                self.verified_transition
                    .as_ref()
                    .map_or(Some(0), |transition| transition.owned_heap_bytes())?,
            )?
            .checked_add(
                self.selected_head_replay
                    .as_ref()
                    .map_or(Some(0), |replay| replay.owned_heap_bytes())?,
            )?
            .checked_add(
                self.prepared_effective_heads
                    .as_ref()
                    .map_or(Some(0), |prepared| prepared.owned_heap_bytes())?,
            )
    }
}
