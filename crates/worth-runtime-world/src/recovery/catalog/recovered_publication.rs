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
    /// An undecided recovered creation also fences durable open attempts, so
    /// its recorded source cannot move before its decision.
    pub(crate) fn has_recovered_creation(&self) -> bool {
        let state = self.locked_state();
        (0..state.slots.span()).any(|position| {
            matches!(
                state.slots.at(position),
                Some((_, RecoveryEntry::RecoveredCreation(_)))
            )
        })
    }
    /// Every undecided recovered original fences fresh work that issues
    /// identities outside a durable open attempt (creations, plain publications).
    pub(crate) fn has_recovered(&self) -> bool {
        let state = self.locked_state();
        (0..state.slots.span()).any(|position| {
            matches!(
                state.slots.at(position),
                Some((
                    _,
                    RecoveryEntry::Recovered(_) | RecoveryEntry::RecoveredCreation(_)
                ))
            )
        })
    }
    pub(crate) fn claim_recovered(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<WorldRecoveredUnpublishedEffects, crate::recovery::RuntimeWorldRecoveryDenial> {
        use crate::recovery::RuntimeWorldRecoveryDenial as Denial;
        self.admit_handle(handle)?;
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
        let source = match &*record.native.lock().unwrap_or_else(|e| e.into_inner()) {
            crate::recovery::RecoveredNativeCustody::OpenOnly => None,
            crate::recovery::RecoveredNativeCustody::Relational(native) => Some(
                crate::recovery::recovered_publication::RecoveredNativeObservation::original(
                    native.clone(),
                ),
            ),
            crate::recovery::RecoveredNativeCustody::Prepared(record) => Some(
                crate::recovery::recovered_publication::RecoveredNativeObservation::prepared(
                    record.clone(),
                ),
            ),
            crate::recovery::RecoveredNativeCustody::Continuing(_)
            | crate::recovery::RecoveredNativeCustody::Working => None,
            crate::recovery::RecoveredNativeCustody::Released => {
                unreachable!("current recovered catalog entry")
            }
        };
        Ok(WorldRecoveredUnpublishedEffects {
            source,
            record: crate::recovery::recovered_publication::SharedRecoveredWorldPublicationRecord::from_arc(record),
            catalog: self.clone(),
        })
    }

    pub(crate) fn admits_recovered(&self, claim: &WorldRecoveredUnpublishedEffects) -> bool {
        if claim.catalog.affinity() != self.affinity() {
            return false;
        }
        let state = self.locked_state();
        matches!(state.slots.get(&claim.record.identity), Some(RecoveryEntry::Recovered(record))
            if std::ptr::eq(Arc::as_ptr(record), &*claim.record))
    }

    pub(crate) fn remove_recovered(&self, record: &RecoveredWorldPublicationRecord) {
        let removed = {
            let mut state = self.locked_state();
            assert!(
                matches!(state.slots.get(&record.identity), Some(RecoveryEntry::Recovered(actual))
                if std::ptr::eq(Arc::as_ptr(actual), record))
            );
            let removed = state.slots.remove(&record.identity);
            state.metadata_bytes -= record.metadata_bytes;
            state.costs.records_cleaned = state.costs.records_cleaned.saturating_add(1);
            removed
        };
        drop(removed);
    }
}
