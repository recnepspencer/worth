//! The existing Busy seat becomes Active with the same complete original custody.
use super::*;
use crate::recovery::RuntimeWorldRecoveryDenial as Denial;
impl ReservedProductUnpublishedRecordUpdate {
    pub(crate) fn reactivate_creation(
        mut self,
        operation: crate::lifecycle::owner::RuntimeWorldOperationReservation,
        capacity: crate::lifecycle::owner::ReservedPublicationAttemptCapacity,
    ) -> Result<crate::publication::ActiveAttemptCustody, Denial> {
        let record = self.record_mut().ok_or(Denial::CallerCapabilityLive)?;
        if !record.is_durable_creation() {
            return Err(Denial::MissingRecord);
        }
        let bytes = record.metadata_bytes();
        let catalog = self.catalog.clone();
        let mut state = catalog.locked_state();
        let slots = state
            .reserved_slots
            .checked_add(1)
            .ok_or(Denial::OutputCapacityExhausted)?;
        let reserved_bytes = state
            .reserved_metadata_bytes
            .checked_add(bytes)
            .ok_or(Denial::OutputCapacityExhausted)?;
        let original = self.record.take().expect("exclusive original");
        let original = Arc::try_unwrap(original)
            .unwrap_or_else(|_| unreachable!("unique original checked before handoff"));
        let active = original.into_active_creation(operation, capacity);
        let removed = state
            .slots
            .replace(&self.identity, RecoveryEntry::Active(active.clone()));
        assert!(matches!(removed, RecoveryEntry::Busy));
        state.updating_slots -= 1;
        state.metadata_bytes -= bytes;
        state.reserved_slots = slots;
        state.reserved_metadata_bytes = reserved_bytes;
        self.armed = false;
        drop(state);
        let slot = ReservedProductUnpublishedSlot {
            catalog,
            reserved_metadata_bytes: bytes,
            armed: true,
            recovered: None,
        };
        Ok(crate::publication::ActiveAttemptCustody::from_reactivated_creation(active, slot))
    }
}
