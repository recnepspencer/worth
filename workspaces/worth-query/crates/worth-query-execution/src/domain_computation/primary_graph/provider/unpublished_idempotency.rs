use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

use super::{WorthQueryProductIdempotencyAffinity, WorthQueryProviderIdempotencyResolution};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

mod reservation;
#[cfg(all(test, feature = "test-world-operation-control"))]
pub(in crate::domain_computation::primary_graph) use reservation::unwind_recovery_inspection_count;
pub(in crate::domain_computation::primary_graph) use reservation::WorthQueryUnpublishedIdempotencyReservation;
#[cfg(test)]
mod tests;

type WorthQueryUnpublishedIdempotencyKey = (WorthQueryProductIdempotencyAffinity, [u8; 32]);

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorthQueryUnpublishedIdempotencyPosture {
    Active,
    Retained,
}

struct WorthQueryUnpublishedIdempotencyEntry {
    binding: WorthQueryApplicationIdempotencyBinding,
    recovery_handle: ProductUnpublishedRecoveryHandle,
    posture: WorthQueryUnpublishedIdempotencyPosture,
}

/// Provider-owned bounded evidence that owner effects were not published as a
/// World product occurrence. Capacity and the exact World recovery identity
/// are bound before owner effects begin.
pub(super) struct WorthQueryUnpublishedIdempotencyStore {
    maximum_entries: usize,
    by_key: BTreeMap<WorthQueryUnpublishedIdempotencyKey, WorthQueryUnpublishedIdempotencyEntry>,
    by_recovery: HashMap<ProductUnpublishedRecoveryHandle, WorthQueryUnpublishedIdempotencyKey>,
}

#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryUnpublishedIdempotencyDisposition {
    store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
}

impl WorthQueryUnpublishedIdempotencyStore {
    pub(super) fn new(maximum_entries: usize) -> Self {
        assert!(
            maximum_entries > 0,
            "unpublished idempotency capacity must be nonzero"
        );
        Self {
            maximum_entries,
            by_key: BTreeMap::new(),
            by_recovery: HashMap::new(),
        }
    }

    fn reserve(
        &mut self,
        product: WorthQueryProductIdempotencyAffinity,
        binding: WorthQueryApplicationIdempotencyBinding,
        recovery_handle: ProductUnpublishedRecoveryHandle,
    ) -> Result<WorthQueryUnpublishedIdempotencyKey, ()> {
        let key = (product, *binding.key_identity());
        if self.by_key.contains_key(&key)
            || self.by_recovery.contains_key(&recovery_handle)
            || self.by_key.len() >= self.maximum_entries
        {
            return Err(());
        }
        self.by_recovery
            .insert(recovery_handle.clone(), key.clone());
        self.by_key.insert(
            key.clone(),
            WorthQueryUnpublishedIdempotencyEntry {
                binding,
                recovery_handle,
                posture: WorthQueryUnpublishedIdempotencyPosture::Active,
            },
        );
        Ok(key)
    }

    fn release_exact(
        &mut self,
        key: &WorthQueryUnpublishedIdempotencyKey,
        handle: &ProductUnpublishedRecoveryHandle,
    ) {
        let matches = self
            .by_key
            .get(key)
            .is_some_and(|entry| &entry.recovery_handle == handle);
        if !matches {
            return;
        }
        self.by_key.remove(key);
        self.by_recovery.remove(handle);
    }

    fn retain_exact(
        &mut self,
        key: &WorthQueryUnpublishedIdempotencyKey,
        handle: &ProductUnpublishedRecoveryHandle,
    ) {
        if let Some(entry) = self.by_key.get_mut(key) {
            if &entry.recovery_handle == handle {
                entry.posture = WorthQueryUnpublishedIdempotencyPosture::Retained;
            }
        }
    }

    fn release_recovery(&mut self, handle: &ProductUnpublishedRecoveryHandle) {
        let Some(key) = self.by_recovery.get(handle).cloned() else {
            return;
        };
        self.release_exact(&key, handle);
    }

    fn resolve(
        &self,
        product: &WorthQueryProductIdempotencyAffinity,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Option<WorthQueryProviderIdempotencyResolution> {
        let recorded = self
            .by_key
            .get(&(product.clone(), *binding.key_identity()))?;
        Some(if recorded.binding == binding {
            WorthQueryProviderIdempotencyResolution::Unpublished
        } else {
            WorthQueryProviderIdempotencyResolution::Drift
        })
    }

    #[cfg(test)]
    fn retained_count(&self) -> usize {
        self.by_key.len()
    }
}

impl WorthQueryUnpublishedIdempotencyDisposition {
    pub(super) fn new(store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>) -> Self {
        Self { store }
    }

    pub(in crate::domain_computation) fn release(&self, handle: &ProductUnpublishedRecoveryHandle) {
        self.store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .release_recovery(handle);
    }
}

impl super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn reserve_unpublished_application_idempotency(
        &self,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        binding: WorthQueryApplicationIdempotencyBinding,
        recovery_handle: ProductUnpublishedRecoveryHandle,
        recovery: worth_runtime_world::facade::RuntimeWorldRecoveryPort,
    ) -> Result<WorthQueryUnpublishedIdempotencyReservation, ()> {
        let product = WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
        let key = self
            .unpublished_idempotency
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reserve(product, binding, recovery_handle.clone())?;
        Ok(WorthQueryUnpublishedIdempotencyReservation::new(
            Arc::clone(&self.unpublished_idempotency),
            key,
            recovery_handle,
            recovery,
        ))
    }

    pub(in crate::domain_computation) fn unpublished_idempotency_disposition(
        &self,
    ) -> WorthQueryUnpublishedIdempotencyDisposition {
        WorthQueryUnpublishedIdempotencyDisposition::new(Arc::clone(&self.unpublished_idempotency))
    }

    pub(super) fn resolve_unpublished_application_idempotency(
        &self,
        product: &WorthQueryProductIdempotencyAffinity,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Option<WorthQueryProviderIdempotencyResolution> {
        self.unpublished_idempotency
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .resolve(product, binding)
    }

    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn unpublished_idempotency_count(
        &self,
    ) -> usize {
        self.unpublished_idempotency
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retained_count()
    }
}
