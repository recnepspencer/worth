//! Live Query preparation readmission never guesses a Native or World result.
use super::WorthQueryProductPreparationPending;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryDenial};
impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(in crate::domain_computation) fn pending_preparation_resolution(
        &self,
        handle: ProductUnpublishedRecoveryHandle,
        stage: super::WorthQueryProductPreparationStage,
    ) -> WorthQueryProductPreparationPending {
        WorthQueryProductPreparationPending::new(
            stage,
            handle,
            self.runtime.product_runtime.owner.recovery_port(),
            self.runtime
                .primary_provider
                .unpublished_idempotency_disposition(),
        )
    }
    pub fn readmit_product_preparation_pending(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<WorthQueryProductPreparationPending, RuntimeWorldRecoveryDenial> {
        let disposition = self
            .runtime
            .primary_provider
            .unpublished_idempotency_disposition();
        let stage = disposition
            .preparation_stage(handle)
            .ok_or(RuntimeWorldRecoveryDenial::MissingRecord)?;
        Ok(WorthQueryProductPreparationPending::new(
            stage,
            handle.clone(),
            self.runtime.product_runtime.owner.recovery_port(),
            disposition,
        ))
    }
}
