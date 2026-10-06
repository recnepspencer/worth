use std::sync::{Arc, Mutex};

use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;

use super::{
    WorthQueryUnpublishedIdempotencyKey, WorthQueryUnpublishedIdempotencyPosture as Posture,
    WorthQueryUnpublishedIdempotencyStore,
};

/// Pins the provider's exact partial against a managed checkout while World
/// performs release. No provider mutex is held across World calls.
pub(in crate::domain_computation) struct WorthQueryUnpublishedReleaseReservation {
    store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
    key: WorthQueryUnpublishedIdempotencyKey,
    handle: ProductUnpublishedRecoveryHandle,
    armed: bool,
}

impl WorthQueryUnpublishedReleaseReservation {
    pub(super) fn begin(
        store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<Option<Self>, ()> {
        let owner = store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(key) = owner.recovery_key(handle).cloned() else {
            return Ok(None);
        };
        let entry = owner.by_key.get(&key).ok_or(())?;
        let mut entry = entry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if entry.recovery_handle != *handle || entry.posture != Posture::Retained {
            return Err(());
        }
        entry.posture = Posture::Releasing;
        drop(entry);
        drop(owner);
        Ok(Some(Self {
            store,
            key,
            handle: handle.clone(),
            armed: true,
        }))
    }

    pub(in crate::domain_computation) fn finish(mut self) {
        let removed = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .release_exact(&self.key, &self.handle);
        self.armed = false;
        drop(removed);
    }
}

impl Drop for WorthQueryUnpublishedReleaseReservation {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let owner = self
            .store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(entry) = owner.by_key.get(&self.key) {
            let mut entry = entry
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if entry.recovery_handle == self.handle && entry.posture == Posture::Releasing {
                entry.posture = Posture::Retained;
            }
        }
    }
}
