//! Real subject/index admission before any World open can exist.
use super::*;

pub(in crate::domain_computation::primary_graph::provider) struct BeforeWorldQueryReservation {
    store: Arc<Mutex<WorthQueryUnpublishedIdempotencyStore>>,
    key: Option<WorthQueryUnpublishedIdempotencyKey>,
    entry: Arc<Mutex<WorthQueryUnpublishedIdempotencyEntry>>,
    slot: Arc<OnceLock<ProductUnpublishedRecoveryHandle>>,
    armed: bool,
}
impl BeforeWorldQueryReservation {
    pub(in crate::domain_computation::primary_graph::provider) fn bind_world(
        mut self,
        handle: ProductUnpublishedRecoveryHandle,
        recovery: worth_runtime_world::facade::RuntimeWorldRecoveryPort,
    ) -> WorthQueryUnpublishedIdempotencyReservation {
        // Only this consuming guard can fill the preinserted canonical row.
        // Handle cloning copies fixed identity data; no allocation or lock.
        self.slot
            .set(handle.clone())
            .expect("one actual World construction binds once");
        let reservation = WorthQueryUnpublishedIdempotencyReservation::new(
            Arc::clone(&self.store),
            self.key.take().expect("original staged key"),
            Arc::clone(&self.entry),
            handle,
            recovery,
        );
        self.armed = false;
        reservation
    }
}
impl Drop for BeforeWorldQueryReservation {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let Some(key) = self.key.as_ref() else {
            return;
        };
        let removed = {
            let mut store = self
                .store
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if self.slot.get().is_none()
                && store
                    .by_key
                    .get(key)
                    .is_some_and(|entry| Arc::ptr_eq(entry, &self.entry))
            {
                store
                    .by_recovery
                    .retain(|(slot, _)| !Arc::ptr_eq(slot, &self.slot));
                store.by_key.remove(key)
            } else {
                None
            }
        };
        drop(removed);
    }
}
impl super::super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph::provider) fn reserve_query_before_world(
        &self,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        binding: WorthQueryApplicationIdempotencyBinding,
    ) -> Result<BeforeWorldQueryReservation, ()> {
        let affinity =
            WorthQueryProductIdempotencyAffinity::from_observation(product.observation());
        let (key, entry, slot) = self
            .unpublished_idempotency
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reserve_before_world(affinity, binding)?;
        Ok(BeforeWorldQueryReservation {
            store: Arc::clone(&self.unpublished_idempotency),
            key: Some(key),
            entry,
            slot,
            armed: true,
        })
    }
}
