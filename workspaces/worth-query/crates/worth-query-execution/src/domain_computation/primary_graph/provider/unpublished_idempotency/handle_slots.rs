//! Existing recovery aliases share each actual one-shot handle slot.
use super::*;
impl WorthQueryUnpublishedIdempotencyEntry {
    pub(super) fn handle(&self) -> &ProductUnpublishedRecoveryHandle {
        self.handle_slot
            .get()
            .expect("bound entry has its actual World handle")
    }
}
impl WorthQueryUnpublishedIdempotencyStore {
    pub(super) fn recovery_key(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Option<&WorthQueryUnpublishedIdempotencyKey> {
        self.by_recovery
            .iter()
            .find(|(candidate, _)| candidate.get() == Some(handle))
            .map(|(_, key)| key)
    }

    pub(super) fn remove_recovery(
        &mut self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Option<WorthQueryUnpublishedIdempotencyKey> {
        let index = self
            .by_recovery
            .iter()
            .position(|(candidate, _)| candidate.get() == Some(handle))?;
        Some(self.by_recovery.swap_remove(index).1)
    }

    pub(super) fn maximum_recovery_mappings(&self) -> Option<usize> {
        self.maximum_entries
            .and_then(|maximum| maximum.checked_mul(2))
    }

    pub(super) fn slot_for(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Arc<OnceLock<ProductUnpublishedRecoveryHandle>> {
        self.by_recovery
            .iter()
            .find(|(slot, _)| slot.get() == Some(handle))
            .map(|(slot, _)| Arc::clone(slot))
            .expect("successor uses its actual preissued slot")
    }
}

impl super::super::WorthQueryPrimaryGraphProvider {
    /// Read-only exact recovery-index observation; terminal key meaning is separate.
    pub fn retains_unpublished_recovery_for_test(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> bool {
        self.unpublished_idempotency
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .recovery_key(handle)
            .is_some()
    }
}
