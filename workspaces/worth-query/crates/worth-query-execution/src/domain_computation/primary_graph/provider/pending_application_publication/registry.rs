//! Pre-effect reservation and unwind-safe custody for pending publications.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use worth_runtime_world::facade::ProductBranchIncarnation;

use super::WorthQueryPendingApplicationPublication;
use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;

pub(in crate::domain_computation::primary_graph) type WorthQueryPendingApplicationPublicationRegistryOwner =
    Arc<Mutex<WorthQueryPendingApplicationPublicationRegistry>>;

pub(in crate::domain_computation::primary_graph) struct WorthQueryPendingApplicationPublicationRegistry
{
    maximum_slots: usize,
    slots: BTreeMap<ProductBranchIncarnation, Arc<WorthQueryApplicationPublicationRecoverySlot>>,
    corrections: BTreeMap<[u8; 32], ProductBranchIncarnation>,
}

pub(in crate::domain_computation::primary_graph) struct WorthQueryApplicationPublicationRecoveryReservation
{
    registry: Arc<Mutex<WorthQueryPendingApplicationPublicationRegistry>>,
    occurrence: ProductBranchIncarnation,
    slot: Arc<WorthQueryApplicationPublicationRecoverySlot>,
    active: bool,
}

pub(super) struct WorthQueryApplicationPublicationRecoverySlot {
    binding: Option<WorthQueryApplicationIdempotencyBinding>,
    state: Mutex<WorthQueryApplicationPublicationRecoveryState>,
}

enum WorthQueryApplicationPublicationRecoveryState {
    Reserved,
    Pending(WorthQueryPendingApplicationPublication),
    Complete,
}

#[cfg(test)]
mod correction_tests;

impl super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn inspect_pending_correction_idempotency(
        &self,
        key: &[u8; 32],
    ) -> Option<WorthQueryApplicationIdempotencyBinding> {
        let slot = self
            .pending_application_publications
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .correction_slot(key)?;
        match slot.inspect_idempotency()? {
            WorthQueryPendingApplicationIdempotency::Pending(binding) => Some(binding),
            WorthQueryPendingApplicationIdempotency::Reserved => None,
        }
    }
}

pub(super) enum WorthQueryPendingApplicationIdempotency {
    Reserved,
    Pending(
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding,
    ),
}

impl WorthQueryPendingApplicationPublicationRegistry {
    pub(in crate::domain_computation::primary_graph) fn new(maximum_slots: usize) -> Self {
        Self {
            maximum_slots,
            slots: BTreeMap::new(),
            corrections: BTreeMap::new(),
        }
    }

    pub(super) fn reserve(
        registry: &Arc<Mutex<Self>>,
        occurrence: ProductBranchIncarnation,
        binding: Option<WorthQueryApplicationIdempotencyBinding>,
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
        let correction = binding
            .filter(|binding| binding.correction_identity().is_some())
            .map(|binding| *binding.key_identity());
        if correction.is_some_and(|key| locked.corrections.contains_key(&key)) {
            return Err("correction command already retains application publication recovery");
        }
        let slot = Arc::new(WorthQueryApplicationPublicationRecoverySlot {
            binding,
            state: Mutex::new(WorthQueryApplicationPublicationRecoveryState::Reserved),
        });
        locked.slots.insert(occurrence, Arc::clone(&slot));
        if let Some(key) = correction {
            locked.corrections.insert(key, occurrence);
        }
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
    pub(super) fn correction_slot(
        &self,
        key: &[u8; 32],
    ) -> Option<Arc<WorthQueryApplicationPublicationRecoverySlot>> {
        self.slot(*self.corrections.get(key)?)
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
            if let Some(binding) = slot
                .binding
                .filter(|binding| binding.correction_identity().is_some())
                && self.corrections.get(binding.key_identity()) == Some(&occurrence)
            {
                self.corrections.remove(binding.key_identity());
            }
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
        if let Some(binding) = self.slot.binding {
            assert_eq!(binding, pending.idempotency());
        }
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
    pub(super) fn inspect_idempotency(&self) -> Option<WorthQueryPendingApplicationIdempotency> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match &*state {
            WorthQueryApplicationPublicationRecoveryState::Reserved => Some(match self.binding {
                Some(binding) => WorthQueryPendingApplicationIdempotency::Pending(binding),
                None => WorthQueryPendingApplicationIdempotency::Reserved,
            }),
            WorthQueryApplicationPublicationRecoveryState::Pending(pending) => Some(
                WorthQueryPendingApplicationIdempotency::Pending(pending.idempotency()),
            ),
            WorthQueryApplicationPublicationRecoveryState::Complete => None,
        }
    }

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
