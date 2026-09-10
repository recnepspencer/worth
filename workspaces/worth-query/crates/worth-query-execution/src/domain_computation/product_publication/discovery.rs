use super::{
    recovery::recovery_failure, WorthQueryProductUnpublishedRecovery,
    WorthQueryProductUnpublishedRecoveryReleaseDenial,
    WorthQueryProductUnpublishedRecoveryReleaseFailure,
};
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
            self.primary_provider.unpublished_idempotency_disposition(),
        ))
    }

    /// Releases one World recovery record only after reserving bounded Query
    /// cleanup custody. No raw owner-retirement work crosses this facade.
    pub fn release_product_publication_recovery(
        &self,
        recovery: WorthQueryProductUnpublishedRecovery,
        minimum_age_ticks: u64,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        WorthQueryProductUnpublishedRecoveryReleaseFailure,
    >{
        let handle = recovery.record_handle().clone();
        let effects = match self
            .product_runtime
            .owner
            .recovery_port()
            .inspect_effects(&handle)
        {
            Ok(effects) => effects,
            Err(denial) => return Err(recovery_failure(recovery, denial.into())),
        };
        let reservation = if effects.destination_branch().is_some() {
            self.product_runtime.reserve_owner_cleanup_for_creation()
        } else {
            self.product_runtime.reserve_owner_cleanup_for_application()
        };
        drop(effects);
        let reservation = match reservation {
            Ok(reservation) => reservation,
            Err(_) => {
                return Err(recovery_failure(
                    recovery,
                    WorthQueryProductUnpublishedRecoveryReleaseDenial::CleanupCapacityExhausted,
                ))
            }
        };
        let (handle, recovery_port, disposition) = recovery.into_release_parts();
        let cleanup = match recovery_port.release_effects(&handle, minimum_age_ticks) {
            Ok(cleanup) => cleanup,
            Err(denial) => {
                return Err(recovery_failure(
                    WorthQueryProductUnpublishedRecovery::new(handle, recovery_port, disposition),
                    denial.into(),
                ))
            }
        };
        disposition.release(&handle);
        let cleanup_identity = reservation.install_unpublished(cleanup);
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanup::new(
            self.product_runtime.clone(),
            cleanup_identity,
        )
        .retry()
        .map_err(WorthQueryProductUnpublishedRecoveryReleaseFailure::OwnerCleanup)
    }
}
