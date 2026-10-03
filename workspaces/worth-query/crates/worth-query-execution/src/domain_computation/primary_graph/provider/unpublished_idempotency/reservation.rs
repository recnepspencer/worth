use std::sync::{Arc, Mutex};

use worth_runtime_world::facade::{
    ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryDenial,
    RuntimeWorldRecoveryPort,
};

use super::{
    WorthQueryUnpublishedIdempotencyEntry, WorthQueryUnpublishedIdempotencyKey,
    WorthQueryUnpublishedIdempotencyPosture, WorthQueryUnpublishedIdempotencyStore,
};
use crate::domain_computation::primary_graph::provider::ManagedUnpublishedAttempt;

/// Linear Query custody paired with World's already-reserved recovery slot.
/// A returned terminal consumes it explicitly; unwind reconciles this one
/// opaque handle after World has restored or retained its own custody.
pub(in crate::domain_computation::primary_graph) struct WorthQueryUnpublishedIdempotencyReservation
{
    store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
    key: WorthQueryUnpublishedIdempotencyKey,
    entry: Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>,
    recovery_handle: ProductUnpublishedRecoveryHandle,
    recovery: RuntimeWorldRecoveryPort,
    armed: bool,
}

impl WorthQueryUnpublishedIdempotencyReservation {
    pub(super) fn new(
        store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
        key: WorthQueryUnpublishedIdempotencyKey,
        entry: Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>,
        recovery_handle: ProductUnpublishedRecoveryHandle,
        recovery: RuntimeWorldRecoveryPort,
    ) -> Self {
        Self {
            store,
            key,
            entry,
            recovery_handle,
            recovery,
            armed: true,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn release(mut self) {
        let removed = self.with_store(|store, key, handle| store.release_exact(key, handle));
        self.armed = false;
        drop(removed);
    }

    pub(in crate::domain_computation::primary_graph) fn retain(
        mut self,
        effects: &ProductUnpublishedOwnerEffects,
    ) {
        let returned_handle = effects.recovery_handle();
        assert_eq!(
            self.recovery_handle, returned_handle,
            "World returned the recovery identity preissued to this publication"
        );
        self.with_store(|store, key, handle| store.retain_exact(key, handle));
        self.armed = false;
    }

    pub(in crate::domain_computation::primary_graph::provider) fn retain_managed(
        mut self,
        returned_handle: ProductUnpublishedRecoveryHandle,
        managed: ManagedUnpublishedAttempt,
    ) {
        assert_eq!(self.recovery_handle, returned_handle);
        {
            let mut entry = self
                .entry
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            assert_eq!(entry.recovery_handle, returned_handle);
            assert!(entry.managed.replace(managed).is_none());
            entry.posture = WorthQueryUnpublishedIdempotencyPosture::Retained;
        }
        self.armed = false;
    }

    fn with_store<R>(
        &self,
        update: impl FnOnce(
            &mut WorthQueryUnpublishedIdempotencyStore,
            &WorthQueryUnpublishedIdempotencyKey,
            &ProductUnpublishedRecoveryHandle,
        ) -> R,
    ) -> R {
        let mut store = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        update(&mut store, &self.key, &self.recovery_handle)
    }
}

impl Drop for WorthQueryUnpublishedIdempotencyReservation {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        #[cfg(all(test, feature = "test-world-operation-control"))]
        UNWIND_RECOVERY_INSPECTIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let retained = !matches!(
            self.recovery.inspect_effects(&self.recovery_handle),
            Err(RuntimeWorldRecoveryDenial::MissingRecord)
        );
        let removed = self.with_store(|store, key, handle| {
            if retained {
                store.retain_exact(key, handle);
                None
            } else {
                store.release_exact(key, handle)
            }
        });
        self.armed = false;
        drop(removed);
    }
}

#[cfg(all(test, feature = "test-world-operation-control"))]
static UNWIND_RECOVERY_INSPECTIONS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

#[cfg(all(test, feature = "test-world-operation-control"))]
pub(in crate::domain_computation::primary_graph) fn unwind_recovery_inspection_count() -> u64 {
    UNWIND_RECOVERY_INSPECTIONS.load(std::sync::atomic::Ordering::Relaxed)
}
