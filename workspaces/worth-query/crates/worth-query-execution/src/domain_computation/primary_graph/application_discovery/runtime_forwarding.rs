//! Thin forwarding shims for `discovery` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_product_publication_discovery {
    use crate::domain_computation::execution_runtime::product_world::OwnerCleanupRetirements;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use std::num::NonZeroUsize;
    use worth_query_execution_query::facade::domain_computation::product_publication::WorthQueryProductUnpublishedRecovery;
    use worth_query_execution_query::facade::domain_computation::product_publication::WorthQueryProductUnpublishedRecoveryReleaseFailure;
    use worth_query_installation::facade::ApplicationSchema;
    use worth_runtime_world::facade::ProductUnpublishedRecoveryHandle;
    use worth_runtime_world::facade::RuntimeWorldRecoveryCursor;
    use worth_runtime_world::facade::RuntimeWorldRecoveryDenial;
    use worth_runtime_world::facade::RuntimeWorldRecoveryPage;
    use worth_runtime_world::facade::RuntimeWorldServiceDenial;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn product_publication_recovery_page(
            &self,
            after: Option<&RuntimeWorldRecoveryCursor>,
            maximum: NonZeroUsize,
        ) -> Result<RuntimeWorldRecoveryPage, RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>>
        {
            self.inner.product_publication_recovery_page(after, maximum)
        }
        pub fn readmit_product_publication_recovery(
            &self,
            handle: &ProductUnpublishedRecoveryHandle,
        ) -> Result<WorthQueryProductUnpublishedRecovery, RuntimeWorldRecoveryDenial> {
            self.inner.readmit_product_publication_recovery(handle)
        }
        pub fn release_product_publication_recovery(&self, recovery: WorthQueryProductUnpublishedRecovery, minimum_age_ticks: u64) -> Result<
            crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
            WorthQueryProductUnpublishedRecoveryReleaseFailure,
        >{
            self.inner
                .release_product_publication_recovery(recovery, minimum_age_ticks)
        }
        pub fn release_product_publication_recovery_in_advancement(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, recovery: WorthQueryProductUnpublishedRecovery, minimum_age_ticks: u64) -> Result<crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt, WorthQueryProductUnpublishedRecoveryReleaseFailure>{
            self.inner
                .release_product_publication_recovery_in_advancement(
                    phase,
                    recovery,
                    minimum_age_ticks,
                )
        }
        pub(crate) fn release_product_recovery_in_request(&self, execution: &worth_execution::ExecutionRequest<'_, '_>, recovery: WorthQueryProductUnpublishedRecovery, minimum_age_ticks: u64, retirements: &mut OwnerCleanupRetirements) -> Result<crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt, WorthQueryProductUnpublishedRecoveryReleaseFailure>{
            self.inner.release_product_recovery_in_request(
                execution,
                recovery,
                minimum_age_ticks,
                retirements,
            )
        }
    }
}
