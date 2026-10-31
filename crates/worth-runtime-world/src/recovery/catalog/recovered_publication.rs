use super::*;
use crate::recovery::{
    RecoveredWorldPublicationInputs, RecoveredWorldPublicationRecord,
    WorldRecoveredUnpublishedEffects,
};

impl ReservedProductUnpublishedSlot {
    pub(crate) fn install_recovered(mut self, inputs: RecoveredWorldPublicationInputs) {
        let record = RecoveredWorldPublicationRecord::new(inputs, self.reserved_metadata_bytes);
        let mut state = self.catalog.locked_state();
        state.slots.insert_recovered(record);
        state.reserved_slots -= 1;
        state.reserved_metadata_bytes -= self.reserved_metadata_bytes;
        state.metadata_bytes += self.reserved_metadata_bytes;
        self.armed = false;
    }
}

impl ProductUnpublishedRecoveryCatalog {
    pub(crate) fn owned_relational_results(
        &self,
    ) -> Vec<Arc<worth_relational::facade::transactions::CommitResult>> {
        let identities = self.identities();
        let mut results = Vec::new();
        for identity in identities {
            let handle = ProductUnpublishedRecoveryHandle::new(identity, self.affinity());
            if let Ok(record) = self.inspect_record(&handle)
                && let Some(result) = record.owned_relational_result()
            {
                results.push(result);
            }
        }
        results
    }

    pub(crate) fn claim_recovered(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<WorldRecoveredUnpublishedEffects, crate::recovery::RuntimeWorldRecoveryDenial> {
        use crate::recovery::RuntimeWorldRecoveryDenial as Denial;
        if handle.catalog_affinity() != self.affinity() {
            return Err(Denial::ForeignHandle);
        }
        let record = {
            let state = self.locked_state();
            match state.slots.get(handle.identity()) {
                Some(RecoveryEntry::Recovered(record)) => record.clone(),
                _ => return Err(Denial::MissingRecord),
            }
        };
        {
            let mut claimed = record.claimed.lock().unwrap_or_else(|e| e.into_inner());
            if *claimed {
                return Err(Denial::Busy);
            }
            *claimed = true;
        }
        Ok(WorldRecoveredUnpublishedEffects {
            record,
            catalog: self.clone(),
        })
    }

    pub(crate) fn admits_recovered(&self, claim: &WorldRecoveredUnpublishedEffects) -> bool {
        if claim.catalog.affinity() != self.affinity() {
            return false;
        }
        let state = self.locked_state();
        matches!(state.slots.get(&claim.record.identity), Some(RecoveryEntry::Recovered(record))
            if Arc::ptr_eq(record, &claim.record))
    }

    pub(crate) fn remove_recovered(&self, record: &Arc<RecoveredWorldPublicationRecord>) {
        let removed = {
            let mut state = self.locked_state();
            assert!(
                matches!(state.slots.get(&record.identity), Some(RecoveryEntry::Recovered(actual))
                if Arc::ptr_eq(actual, record))
            );
            let removed = state.slots.remove(&record.identity);
            state.metadata_bytes -= record.metadata_bytes;
            state.costs.records_cleaned = state.costs.records_cleaned.saturating_add(1);
            removed
        };
        drop(removed);
    }
}
