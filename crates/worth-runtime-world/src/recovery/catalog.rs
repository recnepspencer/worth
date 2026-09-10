use std::sync::{Arc, Mutex, MutexGuard};

use crate::budget::RuntimeWorldBudgetLimit;
use crate::identity::{ProductUnpublishedOwnerEffectsIdentity, RuntimeWorldOwnerIdentity};

use super::product_unpublished::{
    ProductUnpublishedOwnerEffects, ProductUnpublishedOwnerEffectsRecord,
    ProductUnpublishedRecoveryHandle,
};

mod active;
mod slots;
use slots::{RecoveryEntry, RecoveryRecordSlots};
mod initialization;
mod page;
#[path = "catalog/update.rs"]
mod update;
use update::ReservedProductUnpublishedRecordUpdate;

#[cfg(test)]
#[path = "catalog_tests.rs"]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryCatalogDenial {
    ForeignOwner {
        expected: RuntimeWorldOwnerIdentity,
        actual: RuntimeWorldOwnerIdentity,
    },
    CapacityExhausted {
        maximum: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RecoveryRecordRemovalDenial {
    CallerCapabilityLive,
    Busy,
    NotEligible,
}

#[derive(Debug)]
struct RecoveryCatalogState {
    costs: crate::inspection::RuntimeWorldRecoveryCosts,
    owner: RuntimeWorldOwnerIdentity,
    maximum_slots: usize,
    maximum_metadata_bytes: usize,
    reserved_slots: usize,
    abandoned_slots: usize,
    slots: RecoveryRecordSlots,
    reserved_metadata_bytes: usize,
    updating_slots: usize,
    metadata_bytes: usize,
}

/// Runtime World capacity for records whose owner effects outlived product
/// publication. It is a real bounded resource, not copied budget metadata.
#[derive(Debug, Clone)]
pub(crate) struct ProductUnpublishedRecoveryCatalog {
    state: Arc<Mutex<RecoveryCatalogState>>,
}

#[must_use = "an unpublished recovery slot must be retained or dropped"]
pub(crate) struct ReservedProductUnpublishedSlot {
    catalog: ProductUnpublishedRecoveryCatalog,
    reserved_metadata_bytes: usize,
    armed: bool,
}

impl ReservedProductUnpublishedSlot {
    pub(crate) fn recovery_handle(
        &self,
        identity: &crate::identity::ProductUnpublishedOwnerEffectsIdentity,
    ) -> crate::recovery::ProductUnpublishedRecoveryHandle {
        crate::recovery::ProductUnpublishedRecoveryHandle::new(
            identity.clone(),
            self.catalog.affinity(),
        )
    }
}

impl std::fmt::Debug for ReservedProductUnpublishedSlot {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ReservedProductUnpublishedSlot")
            .field("armed", &self.armed)
            .finish_non_exhaustive()
    }
}

impl Drop for ReservedProductUnpublishedSlot {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = self.catalog.locked_state();
        state.reserved_slots = state
            .reserved_slots
            .checked_sub(1)
            .expect("a live recovery reservation owns one slot");
        state.reserved_metadata_bytes = state
            .reserved_metadata_bytes
            .checked_sub(self.reserved_metadata_bytes)
            .expect("a live recovery reservation owns its metadata charge");
        self.armed = false;
    }
}

impl ProductUnpublishedRecoveryCatalog {
    pub(crate) const fn slot_metadata_charge_hint() -> usize {
        RecoveryRecordSlots::metadata_charge_hint()
    }
    fn locked_state(&self) -> MutexGuard<'_, RecoveryCatalogState> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }

    pub(crate) fn affinity(&self) -> usize {
        Arc::as_ptr(&self.state) as usize
    }

    pub(crate) fn reserve_product_unpublished(
        &self,
        owner: RuntimeWorldOwnerIdentity,
    ) -> Result<ReservedProductUnpublishedSlot, RecoveryCatalogDenial> {
        let mut state = self.locked_state();
        if owner != state.owner {
            return Err(RecoveryCatalogDenial::ForeignOwner {
                expected: state.owner,
                actual: owner,
            });
        }
        if state
            .slots
            .retained_len()
            .saturating_add(state.reserved_slots)
            .saturating_add(state.updating_slots)
            .saturating_add(state.abandoned_slots)
            >= state.maximum_slots
        {
            return Err(RecoveryCatalogDenial::CapacityExhausted {
                maximum: state.maximum_slots,
            });
        }
        let metadata_charge = ProductUnpublishedOwnerEffects::metadata_charge_hint();
        let Some(metadata_after) = state
            .metadata_bytes
            .checked_add(state.reserved_metadata_bytes)
            .and_then(|bytes| bytes.checked_add(metadata_charge))
        else {
            return Err(RecoveryCatalogDenial::CapacityExhausted {
                maximum: state.maximum_slots,
            });
        };
        if metadata_after > state.maximum_metadata_bytes {
            return Err(RecoveryCatalogDenial::CapacityExhausted {
                maximum: state.maximum_slots,
            });
        }
        state.reserved_slots += 1;
        state.reserved_metadata_bytes += metadata_charge;
        Ok(ReservedProductUnpublishedSlot {
            catalog: self.clone(),
            reserved_metadata_bytes: metadata_charge,
            armed: true,
        })
    }

    pub(crate) fn lookup_record(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Option<Arc<ProductUnpublishedOwnerEffectsRecord>> {
        self.inspect_record(handle).ok()
    }

    pub(crate) fn inspect_record(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<Arc<ProductUnpublishedOwnerEffectsRecord>, super::RuntimeWorldRecoveryDenial> {
        use super::RuntimeWorldRecoveryDenial;
        if handle.catalog_affinity() != self.affinity() {
            return Err(RuntimeWorldRecoveryDenial::ForeignHandle);
        }
        self.materialize_abandoned(handle.identity());
        let state = self.locked_state();
        if matches!(
            state.slots.get(handle.identity()),
            Some(RecoveryEntry::Busy)
        ) {
            return Err(RuntimeWorldRecoveryDenial::Busy);
        }
        match state.slots.get(handle.identity()) {
            Some(RecoveryEntry::Retained(record)) => Ok(Arc::clone(record)),
            _ => Err(RuntimeWorldRecoveryDenial::MissingRecord),
        }
    }

    pub(crate) fn take_record_for_update(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<ReservedProductUnpublishedRecordUpdate, super::RuntimeWorldRecoveryDenial> {
        if handle.catalog_affinity() != self.affinity() {
            return Err(super::RuntimeWorldRecoveryDenial::ForeignHandle);
        }
        self.materialize_abandoned(handle.identity());
        let mut state = self.locked_state();
        if matches!(
            state.slots.get(handle.identity()),
            Some(RecoveryEntry::Busy)
        ) {
            return Err(super::RuntimeWorldRecoveryDenial::Busy);
        }
        if !matches!(
            state.slots.get(handle.identity()),
            Some(RecoveryEntry::Retained(_))
        ) {
            return Err(super::RuntimeWorldRecoveryDenial::MissingRecord);
        }
        let RecoveryEntry::Retained(record) =
            state.slots.replace(handle.identity(), RecoveryEntry::Busy)
        else {
            unreachable!("checked retained record")
        };
        state.costs.updates_started = state.costs.updates_started.saturating_add(1);
        state.updating_slots = state
            .updating_slots
            .checked_add(1)
            .expect("a bounded recovery update counter cannot overflow");
        Ok(ReservedProductUnpublishedRecordUpdate::new(
            self.clone(),
            handle.identity().clone(),
            record,
        ))
    }

    pub(crate) fn remove_record_if_exclusive<F>(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
        eligible: F,
    ) -> Result<Option<Arc<ProductUnpublishedOwnerEffectsRecord>>, RecoveryRecordRemovalDenial>
    where
        F: FnOnce(&ProductUnpublishedOwnerEffectsRecord) -> bool,
    {
        if handle.catalog_affinity() != self.affinity() {
            return Ok(None);
        }
        self.materialize_abandoned(handle.identity());
        let mut state = self.locked_state();
        if matches!(
            state.slots.get(handle.identity()),
            Some(RecoveryEntry::Busy)
        ) {
            return Err(RecoveryRecordRemovalDenial::Busy);
        }
        let Some(RecoveryEntry::Retained(record)) = state.slots.get(handle.identity()) else {
            return Ok(None);
        };
        if Arc::strong_count(record) != 1 {
            return Err(RecoveryRecordRemovalDenial::CallerCapabilityLive);
        }
        if !eligible(record) {
            return Err(RecoveryRecordRemovalDenial::NotEligible);
        }
        let Some(RecoveryEntry::Retained(record)) = state.slots.remove(handle.identity()) else {
            unreachable!("checked retained record")
        };
        state.metadata_bytes = state
            .metadata_bytes
            .checked_sub(record.metadata_bytes())
            .expect("a catalog record owns its metadata charge");
        state.costs.records_cleaned = state.costs.records_cleaned.saturating_add(1);
        Ok(Some(record))
    }

    pub(crate) fn identities(&self) -> Vec<ProductUnpublishedOwnerEffectsIdentity> {
        let state = self.locked_state();
        state
            .slots
            .iter()
            .filter_map(|(identity, entry)| match entry {
                RecoveryEntry::Active(active) if !active.is_abandoned() => None,
                _ => Some(identity.clone()),
            })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn installed_slots(&self) -> usize {
        let state = self.locked_state();
        state
            .slots
            .retained_len()
            .saturating_add(state.updating_slots)
            .saturating_add(state.abandoned_slots)
    }

    #[cfg(test)]
    pub(crate) fn metadata_bytes(&self) -> usize {
        self.locked_state().metadata_bytes
    }

    pub(crate) fn reserved_slots(&self) -> usize {
        self.locked_state().reserved_slots
    }

    #[cfg(test)]
    pub(crate) fn maximum_slots(&self) -> usize {
        self.locked_state().maximum_slots
    }

    #[cfg(test)]
    #[cfg(feature = "test-operation-control")]
    pub(crate) fn set_metadata_ceiling_for_test(&self, maximum_metadata_bytes: usize) {
        let mut state = self.locked_state();
        assert!(
            state.metadata_bytes <= maximum_metadata_bytes,
            "test metadata ceiling cannot invalidate installed custody"
        );
        state.maximum_metadata_bytes = maximum_metadata_bytes;
    }
}

pub(crate) use ProductUnpublishedRecoveryCatalog as RecoveryCatalog;
