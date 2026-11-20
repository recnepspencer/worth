//! Recovered creations occupy their original custody slot until decided.
use super::*;
use crate::recovery::{
    RecoveredCreationCustody, RecoveredWorldCreationInputs, RecoveredWorldCreationRecord,
    RuntimeWorldRecoveryDenial as Denial,
};

/// One request's exclusive hold on a recovered creation. Dropping it without
/// a terminal decision returns the original custody to the same record.
#[derive(Debug)]
pub(crate) struct ClaimedRecoveredCreation {
    pub(crate) record: Arc<RecoveredWorldCreationRecord>,
    custody: Option<RecoveredCreationCustody>,
    catalog: ProductUnpublishedRecoveryCatalog,
}

impl ReservedProductUnpublishedSlot {
    pub(crate) fn install_recovered_creation(mut self, inputs: RecoveredWorldCreationInputs) {
        let record = RecoveredWorldCreationRecord::new(inputs, self.reserved_metadata_bytes);
        let mut state = self.catalog.locked_state();
        state.slots.insert_recovered_creation(record);
        state.reserved_slots -= 1;
        state.reserved_metadata_bytes -= self.reserved_metadata_bytes;
        state.metadata_bytes += self.reserved_metadata_bytes;
        self.armed = false;
    }
}

impl ProductUnpublishedRecoveryCatalog {
    /// `Ok(None)` when the admitted handle names no recovered creation.
    pub(crate) fn claim_recovered_creation(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<Option<ClaimedRecoveredCreation>, Denial> {
        self.admit_handle(handle)?;
        let record = match self.locked_state().slots.get(handle.identity()) {
            Some(RecoveryEntry::RecoveredCreation(record)) => record.clone(),
            _ => return Ok(None),
        };
        let custody = record.take_custody().ok_or(Denial::Busy)?;
        Ok(Some(ClaimedRecoveredCreation {
            record,
            custody: Some(custody),
            catalog: self.clone(),
        }))
    }
}

impl ClaimedRecoveredCreation {
    pub(crate) fn custody(&mut self) -> &mut RecoveredCreationCustody {
        self.custody.as_mut().expect("claimed original custody")
    }

    /// The decision is durable and installed: the slot leaves the catalog.
    /// The name fence ends only after any destination took its reservation.
    pub(crate) fn finish(mut self) -> RecoveredCreationCustody {
        let custody = self.custody.take().expect("claimed original custody");
        let removed = {
            let mut state = self.catalog.locked_state();
            assert!(matches!(
                state.slots.get(&self.record.identity),
                Some(RecoveryEntry::RecoveredCreation(actual)) if Arc::ptr_eq(actual, &self.record)
            ));
            let removed = state.slots.remove(&self.record.identity);
            state.metadata_bytes -= self.record.metadata_bytes;
            state.costs.records_cleaned = state.costs.records_cleaned.saturating_add(1);
            removed
        };
        drop(removed);
        custody
    }
}

impl Drop for ClaimedRecoveredCreation {
    fn drop(&mut self) {
        if let Some(custody) = self.custody.take() {
            self.record.restore_custody(custody);
        }
    }
}
