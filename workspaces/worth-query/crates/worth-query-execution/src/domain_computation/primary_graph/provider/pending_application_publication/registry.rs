//! Pre-effect reservation and unwind-safe custody for pending publications.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use worth_runtime_world::facade::ProductBranchIncarnation;

use super::WorthQueryPendingApplicationPublication;

pub(in crate::domain_computation::primary_graph) type WorthQueryPendingApplicationPublicationRegistryOwner =
    Arc<Mutex<WorthQueryPendingApplicationPublicationRegistry>>;

pub(in crate::domain_computation::primary_graph) struct WorthQueryPendingApplicationPublicationRegistry
{
    maximum_slots: usize,
    slots: BTreeMap<ProductBranchIncarnation, Arc<WorthQueryApplicationPublicationRecoverySlot>>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationPublicationRecoveryReservation
{
    registry: Arc<Mutex<WorthQueryPendingApplicationPublicationRegistry>>,
    occurrence: ProductBranchIncarnation,
    slot: Arc<WorthQueryApplicationPublicationRecoverySlot>,
    active: bool,
}

pub(super) struct WorthQueryApplicationPublicationRecoverySlot {
    state: Mutex<WorthQueryApplicationPublicationRecoveryState>,
}

enum WorthQueryApplicationPublicationRecoveryState {
    Reserved,
    Pending(WorthQueryPendingApplicationPublication),
    Complete,
}

impl WorthQueryPendingApplicationPublicationRegistry {
    pub(in crate::domain_computation::primary_graph) fn new(maximum_slots: usize) -> Self {
        Self {
            maximum_slots,
            slots: BTreeMap::new(),
        }
    }

    pub(super) fn reserve(
        registry: &Arc<Mutex<Self>>,
        occurrence: ProductBranchIncarnation,
    ) -> Result<WorthQueryApplicationPublicationRecoveryReservation, &'static str> {
        let mut locked = registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if locked.slots.contains_key(&occurrence) {
            return Err("product branch already retains application publication recovery");
        }
        if locked.slots.len() >= locked.maximum_slots {
            return Err("application publication recovery capacity is exhausted");
        }
        let slot = Arc::new(WorthQueryApplicationPublicationRecoverySlot {
            state: Mutex::new(WorthQueryApplicationPublicationRecoveryState::Reserved),
        });
        locked.slots.insert(occurrence, Arc::clone(&slot));
        drop(locked);
        Ok(WorthQueryApplicationPublicationRecoveryReservation {
            registry: Arc::clone(registry),
            occurrence,
            slot,
            active: true,
        })
    }

    pub(super) fn slot(
        &self,
        occurrence: ProductBranchIncarnation,
    ) -> Option<Arc<WorthQueryApplicationPublicationRecoverySlot>> {
        self.slots.get(&occurrence).map(Arc::clone)
    }

    pub(super) fn remove_exact(
        &mut self,
        occurrence: ProductBranchIncarnation,
        slot: &Arc<WorthQueryApplicationPublicationRecoverySlot>,
    ) {
        if self
            .slots
            .get(&occurrence)
            .is_some_and(|installed| Arc::ptr_eq(installed, slot))
        {
            self.slots.remove(&occurrence);
        }
    }

    #[cfg(test)]
    pub(super) fn active_slot_count(&self) -> usize {
        self.slots.len()
    }
}

impl WorthQueryApplicationPublicationRecoveryReservation {
    pub(in crate::domain_computation::primary_graph) fn install(
        mut self,
        pending: WorthQueryPendingApplicationPublication,
    ) {
        assert_eq!(self.occurrence, pending.product_incarnation());
        let mut state = self
            .slot
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(matches!(
            *state,
            WorthQueryApplicationPublicationRecoveryState::Reserved
        ));
        *state = WorthQueryApplicationPublicationRecoveryState::Pending(pending);
        self.active = false;
    }
}

impl Drop for WorthQueryApplicationPublicationRecoveryReservation {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        self.registry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove_exact(self.occurrence, &self.slot);
    }
}

impl WorthQueryApplicationPublicationRecoverySlot {
    pub(super) fn with_pending<Outcome>(
        &self,
        use_pending: impl FnOnce(&mut WorthQueryPendingApplicationPublication) -> Outcome,
    ) -> Result<Outcome, &'static str> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let WorthQueryApplicationPublicationRecoveryState::Pending(pending) = &mut *state else {
            return Err("application publication recovery slot is not pending");
        };
        Ok(use_pending(pending))
    }

    pub(super) fn complete(&self) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(matches!(
            *state,
            WorthQueryApplicationPublicationRecoveryState::Pending(_)
        ));
        *state = WorthQueryApplicationPublicationRecoveryState::Complete;
    }
}
