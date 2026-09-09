use super::WorthQueryProductUnpublishedRecovery;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use std::num::NonZeroUsize;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_world::facade::{
    ProductUnpublishedRecoveryHandle, RuntimeWorldRecoveryCursor, RuntimeWorldRecoveryDenial,
    RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial,
};

impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    /// Bounded live discovery from the same World catalog. Vacancies consume
    /// the work bound, so an empty page can still have a continuation.
    pub fn product_publication_recovery_page(
        &self,
        after: Option<&RuntimeWorldRecoveryCursor>,
        maximum: NonZeroUsize,
    ) -> Result<RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>>
    {
        self.product_runtime
            .owner
            .inspection_port()
            .recovery_page(after, maximum)
    }

    /// A descriptive handle is admitted only by this root's owning World.
    pub fn readmit_product_publication_recovery(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<WorthQueryProductUnpublishedRecovery, RuntimeWorldRecoveryDenial> {
        let recovery = self.product_runtime.owner.recovery_port();
        drop(recovery.inspect_effects(handle)?);
        Ok(WorthQueryProductUnpublishedRecovery::new(
            handle.clone(),
            recovery,
        ))
    }
}
