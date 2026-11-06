//! The original recovered catalog seat rolls back until active custody is complete.
use super::*;
use crate::recovery::{RuntimeWorldRecoveryDenial as Denial, WorldRecoveredUnpublishedEffects};

impl ProductUnpublishedRecoveryCatalog {
    pub(crate) fn reserve_recovered_reactivation(
        &self,
        claim: &WorldRecoveredUnpublishedEffects,
    ) -> Result<ReservedProductUnpublishedSlot, Denial> {
        self.admit_handle(&crate::recovery::ProductUnpublishedRecoveryHandle::new(
            claim.record.identity.clone(),
            claim.catalog.affinity(),
        ))?;
        if claim.catalog.affinity() != self.affinity() {
            return Err(Denial::ForeignHandle);
        }
        let mut state = self.locked_state();
        if !matches!(state.slots.get(&claim.record.identity), Some(RecoveryEntry::Recovered(record)) if Arc::ptr_eq(record, &claim.record))
        {
            return Err(Denial::MissingRecord);
        }
        let reserved_slots = state
            .reserved_slots
            .checked_add(1)
            .ok_or(Denial::OutputCapacityExhausted)?;
        let reserved_bytes = state
            .reserved_metadata_bytes
            .checked_add(claim.record.metadata_bytes)
            .ok_or(Denial::OutputCapacityExhausted)?;
        let removed = state
            .slots
            .replace(&claim.record.identity, RecoveryEntry::Busy);
        state.metadata_bytes -= claim.record.metadata_bytes;
        state.reserved_slots = reserved_slots;
        state.reserved_metadata_bytes = reserved_bytes;
        drop(state);
        drop(removed);
        Ok(ReservedProductUnpublishedSlot {
            catalog: self.clone(),
            reserved_metadata_bytes: claim.record.metadata_bytes,
            armed: true,
            recovered: Some(claim.record.clone()),
        })
    }
}

impl ReservedProductUnpublishedSlot {
    pub(super) fn restore_recovered(&mut self) -> bool {
        let Some(original) = self.recovered.take() else {
            return false;
        };
        let identity = original.identity.clone();
        let mut state = self.catalog.locked_state();
        assert!(matches!(
            state.slots.get(&original.identity),
            Some(RecoveryEntry::Busy)
        ));
        let removed = state
            .slots
            .replace(&identity, RecoveryEntry::Recovered(original));
        state.reserved_slots -= 1;
        state.reserved_metadata_bytes -= self.reserved_metadata_bytes;
        state.metadata_bytes += self.reserved_metadata_bytes;
        self.armed = false;
        drop(state);
        drop(removed);
        true
    }
}
