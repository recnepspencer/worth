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
    pub(in crate::domain_computation) fn validate_original(
        &self,
        binding: &super::super::idempotency::released_original::OriginalBinding,
        expected: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<(), ()> {
        let store = self.store.lock().unwrap_or_else(|e| e.into_inner());
        let entry = store
            .by_key
            .get(&self.key)
            .ok_or(())?
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let affinity =
            super::super::idempotency::WorthQueryProductIdempotencyAffinity::from_observation(
                expected,
            );
        let same = entry
            .binding
            .match_recorded_intent(&binding.binding.intent)
            .map_err(|_| ())?;
        if entry.handle() != &self.handle || entry.binding.key_identity() != &binding.binding.key
            || same != crate::domain_computation::primary_graph::application_attempt::WorthQueryRecordedIntentMatch::Same
            || binding.binding.affinity != super::super::idempotency::recorded_origin::affinity_text(&affinity) {
            return Err(());
        }
        Ok(())
    }
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
        if entry.handle() != handle || entry.posture != Posture::Retained {
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

    pub(in crate::domain_computation) fn finish(
        mut self,
        binding: Option<Arc<super::super::idempotency::released_original::OriginalBinding>>,
        original: worth_runtime_world::facade::WorldReleasedOriginal,
    ) {
        let mut store = self.store.lock().unwrap_or_else(|e| e.into_inner());
        let retired;
        let removed;
        if original.native_semantic_result().is_some() {
            let binding = binding.expect("original binding admitted before effect-bearing release");
            let entry = store
                .by_key
                .get(&self.key)
                .expect("same exclusive provider release");
            let mut entry = entry.lock().unwrap_or_else(|e| e.into_inner());
            assert_eq!(entry.handle(), &self.handle);
            entry.released = Some(
                super::super::idempotency::WorthQueryReleasedApplicationEffect::new(
                    binding, original,
                ),
            );
            entry.posture = Posture::Released;
            retired = (entry.managed.take(), entry.preparing.take());
            removed = None;
            drop(entry);
            store.by_recovery.retain(|(_, key)| key != &self.key);
        } else {
            retired = (None, None);
            removed = store.release_exact(&self.key, &self.handle);
        }
        drop(store);
        drop((retired, removed));
        self.armed = false;
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
            if entry.handle() == &self.handle && entry.posture == Posture::Releasing {
                entry.posture = Posture::Retained;
            }
        }
    }
}
