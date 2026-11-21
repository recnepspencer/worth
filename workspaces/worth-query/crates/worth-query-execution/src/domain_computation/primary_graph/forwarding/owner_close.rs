//! Thin forwarding shims for `owner_close` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_product_publication_owner_close {
    use crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use worth_query_execution_query::facade::domain_computation::product_publication::WorthQueryProductUnpublishedRecovery as Recovery;
    use worth_query_execution_query::facade::domain_computation::product_publication::WorthQueryProductUnpublishedRecoveryReleaseFailure as Failure;
    use worth_query_installation::facade::ApplicationSchema;
    impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
        pub fn close_owned_product_publication_recovery(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            _access: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
            recovery: Recovery,
        ) -> Result<WorthQueryProductBranchOwnerCleanupReceipt, Failure> {
            self.inner
                .close_owned_product_publication_recovery(phase, _access, recovery)
        }
        pub(crate) fn close_product_publication_recovery_in_advancement(
            &self,
            phase: &WorthQueryAdvancementPhase<'_>,
            recovery: Recovery,
        ) -> Result<WorthQueryProductBranchOwnerCleanupReceipt, Failure> {
            self.inner
                .close_product_publication_recovery_in_advancement(phase, recovery)
        }
    }
}
