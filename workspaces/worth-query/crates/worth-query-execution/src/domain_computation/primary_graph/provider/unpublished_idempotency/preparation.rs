//! Exact preparation observation and consuming World abandonment handoff.
use super::*;
impl WorthQueryUnpublishedIdempotencyDisposition {
    pub(in crate::domain_computation) fn abandon_preparing_world(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<(), ()> {
        let phase = {
            let owner = self
                .store
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(key) = owner.recovery_key(handle) else {
                return Err(());
            };
            let entry = owner.by_key.get(key).ok_or(())?;
            let mut entry = entry
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if entry.handle() != handle
                || entry.posture != WorthQueryUnpublishedIdempotencyPosture::Retained
            {
                return Err(());
            }
            entry
                .preparing
                .as_mut()
                .map(|preparation| preparation.abandon_world_phase())
        };
        // World abandonment can call owners and destroy retained values. BOTH
        // Query mutex guards ended before this exact phase is dropped.
        drop(phase);
        Ok(())
    }

    pub(in crate::domain_computation) fn preparation_stage(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Option<crate::domain_computation::WorthQueryProductPreparationStage> {
        let owner = self
            .store
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let key = owner.recovery_key(handle)?;
        let entry = owner
            .by_key
            .get(key)?
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if entry.handle() != handle {
            return None;
        }
        entry
            .preparing
            .as_ref()
            .map(|preparing| preparing.stage())
            .or_else(|| {
                (entry.posture == WorthQueryUnpublishedIdempotencyPosture::Preparing).then_some(
                    crate::domain_computation::WorthQueryProductPreparationStage::CheckedOut,
                )
            })
    }
}
impl super::super::WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn preparation_pending(
        &self,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        handle: ProductUnpublishedRecoveryHandle,
        stage: crate::domain_computation::WorthQueryProductPreparationStage,
    ) -> crate::domain_computation::WorthQueryProductPreparationPending {
        crate::domain_computation::WorthQueryProductPreparationPending::new(
            stage,
            handle,
            product.recovery(),
            self.unpublished_idempotency_disposition(),
        )
    }
}
