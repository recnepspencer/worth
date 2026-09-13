use std::sync::Arc;

use crate::identity::ProductUnpublishedOwnerEffectsIdentity;
use crate::publication::ActiveAttemptRecord;

use super::RecoveryEntry;
use super::{ProductUnpublishedRecoveryCatalog, ReservedProductUnpublishedSlot};

impl ReservedProductUnpublishedSlot {
    /// Choose an explicit retained terminal through the same owner record
    /// and reservation conversion used by caller abandonment.
    pub(crate) fn retain_active(
        mut self,
        identity: &ProductUnpublishedOwnerEffectsIdentity,
    ) -> super::ProductUnpublishedOwnerEffects {
        let (record, removed, permits) = {
            let mut state = self.catalog.locked_state();
            let Some(RecoveryEntry::Active(active)) = state.slots.get(identity) else {
                unreachable!("registered active attempt")
            };
            let permits = active.abandon();
            let record = active
                .materialize_abandoned(self.catalog.affinity())
                .expect("retained active evidence is representable");
            state.costs.retained_records_created =
                state.costs.retained_records_created.saturating_add(1);
            state.reserved_slots -= 1;
            state.reserved_metadata_bytes -= self.reserved_metadata_bytes;
            state.metadata_bytes += self.reserved_metadata_bytes;
            // Acquire the caller's capability before exposing the record to
            // cleanup. Releasing admission first would allow close or another
            // caller to remove the record before this terminal received it.
            let removed = state
                .slots
                .replace(identity, RecoveryEntry::Retained(Arc::clone(&record)));
            self.armed = false;
            (record, removed, permits)
        };
        drop(removed);
        drop(permits);
        super::ProductUnpublishedOwnerEffects::from_catalog_record(record)
    }

    /// Install the owner record while the slot is still reserved, before any
    /// component effect. Live attempts and abandoned records use one budget.
    pub(crate) fn register_active(&self, record: Arc<ActiveAttemptRecord>) {
        let mut state = self.catalog.locked_state();
        assert_eq!(record.identity().owner_identity(), state.owner);
        state.slots.insert_active(record);
    }

    pub(crate) fn remove_active(&self, identity: &ProductUnpublishedOwnerEffectsIdentity) {
        let record = {
            let mut state = self.catalog.locked_state();
            if matches!(state.slots.get(identity), Some(RecoveryEntry::Active(_))) {
                state.slots.remove(identity)
            } else {
                None
            }
        };
        drop(record);
    }

    /// Convert the existing slot charge to retained custody without allocating
    /// a record, installing history, acquiring pins, or contacting an owner.
    pub(crate) fn abandon_active(mut self, identity: &ProductUnpublishedOwnerEffectsIdentity) {
        let permits = {
            let mut state = self.catalog.locked_state();
            let Some(RecoveryEntry::Active(record)) = state.slots.get(identity) else {
                unreachable!("registered active attempt")
            };
            let permits = record.abandon();
            state.costs.retained_records_created =
                state.costs.retained_records_created.saturating_add(1);
            state.reserved_slots -= 1;
            state.reserved_metadata_bytes -= self.reserved_metadata_bytes;
            state.abandoned_slots += 1;
            state.metadata_bytes += self.reserved_metadata_bytes;
            self.armed = false;
            permits
        };
        drop(permits);
    }
}

impl ProductUnpublishedRecoveryCatalog {
    /// Convert under one catalog selection. A competing lookup sees either the
    /// complete original custody or the complete retained row. The slot and
    /// byte charge remain unchanged, and destructors run outside the lock.
    pub(super) fn materialize_abandoned(&self, identity: &ProductUnpublishedOwnerEffectsIdentity) {
        let removed = {
            let mut state = self.locked_state();
            let Some(RecoveryEntry::Active(active)) = state.slots.get(identity) else {
                return;
            };
            let Some(record) = active.materialize_abandoned(self.affinity()) else {
                return;
            };
            state.abandoned_slots -= 1;
            state
                .slots
                .replace(identity, RecoveryEntry::Retained(record))
        };
        drop(removed);
    }
}
