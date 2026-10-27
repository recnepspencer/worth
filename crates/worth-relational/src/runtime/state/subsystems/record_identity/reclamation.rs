//! The allocator lock spans admission, storage reset and reusable-slot handoff.
use super::{ReclaimedRecordSlot, RecordIdentityState, RecordIdentitySubsystem};
use crate::execution::RelationalExecutionDenialCause as Cause;

impl RecordIdentitySubsystem {
    /// The caller holds the actual storage writer. All fallible admission ends
    /// before reset; accepting the same slot then cannot refuse or checkpoint.
    pub(crate) fn reclaim_after_admission(
        &self,
        reclaimed: ReclaimedRecordSlot,
        storage_bytes: u64,
        prepare: &mut impl FnMut(u64, u64) -> Result<(), Cause>,
        reset: impl FnOnce(),
    ) -> Result<(), Cause> {
        let mut state = self.lock();
        let work = (state.reusable.navigation_work_bound() as u64)
            .checked_add(state.next_slots.navigation_work_bound() as u64)
            .and_then(|paths| paths.checked_mul(4))
            .ok_or(Cause::WorkCounterOverflow)?;
        prepare(work, 0)?;
        let bytes = state.reclamation_bound(reclaimed)?;
        prepare(
            0,
            storage_bytes
                .checked_add(bytes)
                .ok_or(Cause::ChargedBytesOverflow)?,
        )?;
        reset();
        state.admit_reclaimed(reclaimed);
        Ok(())
    }
}

impl RecordIdentityState {
    fn reclamation_bound(&self, reclaimed: ReclaimedRecordSlot) -> Result<u64, Cause> {
        let key = (reclaimed.class, reclaimed.partition_id, reclaimed.slot);
        let mut bytes = if self.reusable.contains_key(&key) {
            0
        } else {
            self.reusable
                .insertion_allocation_bytes(&key)
                .ok_or(Cause::ChargedBytesOverflow)?
        };
        if let Some(next) = reclaimed.slot.checked_add(1) {
            let frontier = (reclaimed.class, reclaimed.partition_id);
            if self.next_slots.get(&frontier).is_none_or(|old| *old < next) {
                bytes = bytes
                    .checked_add(
                        self.next_slots
                            .insertion_allocation_bytes(&frontier)
                            .ok_or(Cause::ChargedBytesOverflow)?,
                    )
                    .ok_or(Cause::ChargedBytesOverflow)?;
            }
        }
        Ok(bytes)
    }

    pub(super) fn admit_reclaimed(&mut self, reclaimed: ReclaimedRecordSlot) {
        let key = (reclaimed.class, reclaimed.partition_id, reclaimed.slot);
        if !self.reusable.contains_key(&key) {
            self.reusable.insert(key, ());
        }
        if let Some(next) = reclaimed.slot.checked_add(1) {
            let frontier = (reclaimed.class, reclaimed.partition_id);
            if self.next_slots.get(&frontier).is_none_or(|old| *old < next) {
                self.next_slots.insert(frontier, next);
            }
        }
    }
}
