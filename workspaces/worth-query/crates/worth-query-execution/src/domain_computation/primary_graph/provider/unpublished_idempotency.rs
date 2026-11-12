use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use super::ManagedUnpublishedAttempt;
use super::{WorthQueryProductIdempotencyAffinity, WorthQueryProviderIdempotencyResolution};
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationIdempotencyBinding;
use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

mod before_world;
mod handle_slots;
mod managed_recovery;
pub(super) mod recovered;
pub(in crate::domain_computation::primary_graph::provider) use before_world::BeforeWorldQueryReservation;
mod preparation;
mod release;
pub(crate) mod reservation;
pub(in crate::domain_computation::primary_graph::provider) use managed_recovery::{
    ManagedUnpublishedRecoveryGuard, ManagedUnpublishedRecoveryStop,
};
pub(in crate::domain_computation) use release::WorthQueryUnpublishedReleaseReservation;

pub(in crate::domain_computation) use reservation::QueryDurablePreparationGuard;
pub(in crate::domain_computation::primary_graph) use reservation::WorthQueryUnpublishedIdempotencyReservation;

type WorthQueryUnpublishedIdempotencyKey = super::idempotency::WorthQueryIdempotencyLookupKey;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WorthQueryUnpublishedIdempotencyPosture {
    Active,
    Preparing,
    Retained,
    Recovering,
    Releasing,
    Released,
}

struct WorthQueryUnpublishedIdempotencyEntry {
    binding: WorthQueryApplicationIdempotencyBinding,
    handle_slot: Arc<OnceLock<ProductUnpublishedRecoveryHandle>>,
    posture: WorthQueryUnpublishedIdempotencyPosture,
    managed: Option<ManagedUnpublishedAttempt>,
    preparing: Option<super::RetainedQueryDurablePreparation>,
    released: Option<super::idempotency::WorthQueryReleasedApplicationEffect>,
}

/// Provider-owned bounded evidence that owner effects were not published as a
/// World product occurrence. Capacity and the exact World recovery identity
/// are bound before owner effects begin.
pub(super) struct WorthQueryUnpublishedIdempotencyStore {
    maximum_entries: Option<usize>,
    recovered: BTreeMap<WorthQueryUnpublishedIdempotencyKey, recovered::RecoveredIdempotencyEntry>,
    by_key: BTreeMap<
        WorthQueryUnpublishedIdempotencyKey,
        Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>,
    >,
    by_recovery: Vec<(
        Arc<OnceLock<ProductUnpublishedRecoveryHandle>>,
        WorthQueryUnpublishedIdempotencyKey,
    )>,
}

#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryUnpublishedIdempotencyDisposition {
    store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
}

impl WorthQueryUnpublishedIdempotencyStore {
    pub(super) fn home_close_has_work(&self) -> bool {
        self.by_key.values().any(|entry| {
            entry
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .released
                .is_none()
        }) || !self.by_recovery.is_empty()
    }
    pub(super) fn new(maximum_entries: Option<usize>) -> Self {
        assert!(
            maximum_entries != Some(0),
            "explicit unpublished capacity must be nonzero"
        );
        Self {
            maximum_entries,
            recovered: BTreeMap::new(),
            by_key: BTreeMap::new(),
            by_recovery: Vec::new(),
        }
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
        if self.recovery_key(&recovery_handle).is_some() {
            return Err(());
        }
        let (key, entry, slot) = self.reserve_before_world(product, binding)?;
        slot.set(recovery_handle)
            .expect("one private reservation binds once");
        entry
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .posture = WorthQueryUnpublishedIdempotencyPosture::Active;
        Ok((key, entry))
    }

    fn reserve_before_world(
        &mut self,
        product: WorthQueryProductIdempotencyAffinity,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<
        (
            WorthQueryUnpublishedIdempotencyKey,
            Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>,
            Arc<OnceLock<ProductUnpublishedRecoveryHandle>>,
        ),
        (),
    > {
        let key = WorthQueryUnpublishedIdempotencyKey::for_binding(&product, &binding);
        if self.recovered.contains_key(&key)
            || self.by_key.contains_key(&key)
            || self
                .maximum_entries
                .is_some_and(|maximum| self.by_key.len() + self.recovered.len() >= maximum)
            || match self.maximum_entries {
                Some(_) => self.by_recovery.len() >= self.maximum_recovery_mappings().ok_or(())?,
                None => false,
            }
        {
            return Err(());
        }
        self.by_recovery.try_reserve_exact(1).map_err(|_| ())?;
        let slot = Arc::new(OnceLock::new());
        let entry = Arc::new(Mutex::new(WorthQueryUnpublishedIdempotencyEntry {
            binding,
            handle_slot: Arc::clone(&slot),
            posture: WorthQueryUnpublishedIdempotencyPosture::Preparing,
            managed: None,
            preparing: None,
            released: None,
        }));
        // This is an occupied canonical index row, not spare shared capacity.
        self.by_recovery.push((Arc::clone(&slot), key.clone()));
        self.by_key.insert(key.clone(), Arc::clone(&entry));
        Ok((key, entry, slot))
    }

    fn release_exact(
        &mut self,
        key: &WorthQueryUnpublishedIdempotencyKey,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Option<Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>> {
        let matches = self.by_key.get(key).is_some_and(|entry| {
            entry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .handle_slot
                .get()
                == Some(handle)
        });
        if !matches {
            return None;
        }
        let removed = self.by_key.remove(key);
        self.by_recovery.retain(|(_, candidate)| candidate != key);
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
            if entry.handle_slot.get() == Some(handle) {
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
            .get(&WorthQueryUnpublishedIdempotencyKey::for_binding(
                product, &binding,
            ))?;
        let recorded = recorded
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        Some(if recorded.binding == binding {
            if let Some(original) = &recorded.released {
                WorthQueryProviderIdempotencyResolution::ReleasedOriginal(original.clone())
            } else if recorded.handle_slot.get().is_none() {
                WorthQueryProviderIdempotencyResolution::PreparationBusy
            } else if let Some(preparing) = &recorded.preparing {
                WorthQueryProviderIdempotencyResolution::PreparationPending {
                    handle: recorded.handle().clone(),
                    stage: preparing.stage(),
                }
            } else if recorded.posture == WorthQueryUnpublishedIdempotencyPosture::Preparing {
                WorthQueryProviderIdempotencyResolution::PreparationPending {
                    handle: recorded.handle().clone(),
                    stage: crate::domain_computation::WorthQueryProductPreparationStage::CheckedOut,
                }
            } else {
                WorthQueryProviderIdempotencyResolution::Unpublished
            }
        } else {
            WorthQueryProviderIdempotencyResolution::Drift
        })
    }

    fn inspect_exact(
        &self,
        key: &WorthQueryUnpublishedIdempotencyKey,
    ) -> Option<(
        WorthQueryApplicationIdempotencyBinding,
        ProductUnpublishedRecoveryHandle,
    )> {
        let recorded = self.by_key.get(key)?;
        let recorded = recorded
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if recorded.released.is_some() {
            return None;
        }
        Some((recorded.binding, recorded.handle_slot.get()?.clone()))
    }

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
        recovery: worth_runtime_world::facade::RuntimeWorldRecoveryPort,
    ) -> Result<ManagedUnpublishedRecoveryGuard, ManagedUnpublishedRecoveryStop> {
        ManagedUnpublishedRecoveryGuard::begin(
            Arc::clone(&self.unpublished_idempotency),
            handle,
            recovery,
        )
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
            .inspect_exact(&WorthQueryUnpublishedIdempotencyKey::for_binding(
                product, &binding,
            ))
    }

    pub(in crate::domain_computation::primary_graph::provider) fn inspect_unpublished_key(
        &self,
        key: &WorthQueryUnpublishedIdempotencyKey,
    ) -> Option<(
        WorthQueryApplicationIdempotencyBinding,
        ProductUnpublishedRecoveryHandle,
    )> {
        self.unpublished_idempotency
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .inspect_exact(key)
    }

    pub fn unpublished_idempotency_count(&self) -> usize {
        self.unpublished_idempotency
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .retained_count()
    }
}
