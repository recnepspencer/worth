use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use super::ManagedUnpublishedAttempt;
use super::{WorthQueryProductIdempotencyAffinity, WorthQueryProviderIdempotencyResolution};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

mod managed_recovery;
mod release;
mod reservation;
pub(in crate::domain_computation::primary_graph::provider) use managed_recovery::{
    ManagedUnpublishedRecoveryGuard, ManagedUnpublishedRecoveryStop,
};
pub(in crate::domain_computation) use release::WorthQueryUnpublishedReleaseReservation;
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
    Recovering,
    Releasing,
}

struct WorthQueryUnpublishedIdempotencyEntry {
    binding: WorthQueryApplicationIdempotencyBinding,
    recovery_handle: ProductUnpublishedRecoveryHandle,
    posture: WorthQueryUnpublishedIdempotencyPosture,
    managed: Option<ManagedUnpublishedAttempt>,
}

/// Provider-owned bounded evidence that owner effects were not published as a
/// World product occurrence. Capacity and the exact World recovery identity
/// are bound before owner effects begin.
pub(super) struct WorthQueryUnpublishedIdempotencyStore {
    maximum_entries: usize,
    by_key: BTreeMap<
        WorthQueryUnpublishedIdempotencyKey,
        Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>,
    >,
    by_recovery: Vec<(
        ProductUnpublishedRecoveryHandle,
        WorthQueryUnpublishedIdempotencyKey,
    )>,
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
            by_recovery: Vec::new(),
        }
    }

    fn recovery_key(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Option<&WorthQueryUnpublishedIdempotencyKey> {
        self.by_recovery
            .iter()
            .find(|(candidate, _)| candidate == handle)
            .map(|(_, key)| key)
    }

    fn remove_recovery(
        &mut self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Option<WorthQueryUnpublishedIdempotencyKey> {
        let index = self
            .by_recovery
            .iter()
            .position(|(candidate, _)| candidate == handle)?;
        Some(self.by_recovery.swap_remove(index).1)
    }

    fn maximum_recovery_mappings(&self) -> Option<usize> {
        self.maximum_entries.checked_mul(2)
    }

    fn reserve(
        &mut self,
        product: WorthQueryProductIdempotencyAffinity,
        binding: WorthQueryApplicationIdempotencyBinding,
        recovery_handle: ProductUnpublishedRecoveryHandle,
    ) -> Result<
        (
            WorthQueryUnpublishedIdempotencyKey,
            Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>,
        ),
        (),
    > {
        let key = (product, *binding.key_identity());
        if self.by_key.contains_key(&key)
            || self.recovery_key(&recovery_handle).is_some()
            || self.by_key.len() >= self.maximum_entries
            || self.by_recovery.len() >= self.maximum_recovery_mappings().ok_or(())?
        {
            return Err(());
        }
        self.by_recovery.try_reserve_exact(1).map_err(|_| ())?;
        self.by_recovery
            .push((recovery_handle.clone(), key.clone()));
        let entry = Arc::new(Mutex::new(WorthQueryUnpublishedIdempotencyEntry {
            binding,
            recovery_handle,
            posture: WorthQueryUnpublishedIdempotencyPosture::Active,
            managed: None,
        }));
        self.by_key.insert(key.clone(), Arc::clone(&entry));
        Ok((key, entry))
    }

    fn release_exact(
        &mut self,
        key: &WorthQueryUnpublishedIdempotencyKey,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Option<Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>> {
        let matches = self.by_key.get(key).is_some_and(|entry| {
            &entry
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .recovery_handle
                == handle
        });
        if !matches {
            return None;
        }
        let removed = self.by_key.remove(key);
        self.remove_recovery(handle);
        removed
    }

    fn retain_exact(
        &mut self,
        key: &WorthQueryUnpublishedIdempotencyKey,
        handle: &ProductUnpublishedRecoveryHandle,
    ) {
        if let Some(entry) = self.by_key.get(key) {
            let mut entry = entry
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if &entry.recovery_handle == handle {
                entry.posture = WorthQueryUnpublishedIdempotencyPosture::Retained;
            }
        }
    }

    fn resolve(
        &self,
        product: &WorthQueryProductIdempotencyAffinity,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Option<WorthQueryProviderIdempotencyResolution> {
        let recorded = self
            .by_key
            .get(&(product.clone(), *binding.key_identity()))?;
        let recorded = recorded
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Some(if recorded.binding == binding {
            WorthQueryProviderIdempotencyResolution::Unpublished
        } else {
            WorthQueryProviderIdempotencyResolution::Drift
        })
    }

    fn inspect_exact(
        &self,
        product: &WorthQueryProductIdempotencyAffinity,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Option<(
        WorthQueryApplicationIdempotencyBinding,
        ProductUnpublishedRecoveryHandle,
    )> {
        let recorded = self
            .by_key
            .get(&(product.clone(), *binding.key_identity()))?;
        let recorded = recorded
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Some((recorded.binding, recorded.recovery_handle.clone()))
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

    pub(in crate::domain_computation) fn reserve_release(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<Option<WorthQueryUnpublishedReleaseReservation>, ()> {
        WorthQueryUnpublishedReleaseReservation::begin(Arc::clone(&self.store), handle)
    }
}

impl super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph::provider) fn begin_managed_unpublished_recovery(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<ManagedUnpublishedRecoveryGuard, ManagedUnpublishedRecoveryStop> {
        ManagedUnpublishedRecoveryGuard::begin(Arc::clone(&self.unpublished_idempotency), handle)
    }

    pub(in crate::domain_computation::primary_graph) fn reserve_unpublished_application_idempotency(
        &self,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        binding: WorthQueryApplicationIdempotencyBinding,
        recovery_handle: ProductUnpublishedRecoveryHandle,
        recovery: worth_runtime_world::facade::RuntimeWorldRecoveryPort,
    ) -> Result<WorthQueryUnpublishedIdempotencyReservation, ()> {
        let product = WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
        let (key, entry) = self
            .unpublished_idempotency
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .reserve(product, binding, recovery_handle.clone())?;
        Ok(WorthQueryUnpublishedIdempotencyReservation::new(
            Arc::clone(&self.unpublished_idempotency),
            key,
            entry,
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

    pub(in crate::domain_computation::primary_graph) fn inspect_unpublished_application_idempotency(
        &self,
        product: &WorthQueryProductIdempotencyAffinity,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Option<(
        WorthQueryApplicationIdempotencyBinding,
        ProductUnpublishedRecoveryHandle,
    )> {
        self.unpublished_idempotency
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .inspect_exact(product, binding)
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
