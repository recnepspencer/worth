//! Thin forwarding shims for `recovery_handle` methods onto `worth-query-execution-query`.

#[allow(unused_imports, dead_code)]
mod domain_computation_application_aftermath_recovery_handle_mint {
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_handle::denial::WorthQueryRecoveryHandleDenial;
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_handle::handle::WorthQueryRecoveryHandle;
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_handle::mint::WorthQueryRecoveryClaimStatus;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema, {
        pub fn mint_recovery_handle(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, receipt: &WorthQueryApplicationCommitReceipt) -> Result<WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial> {
            self.inner.mint_recovery_handle(phase, receipt)
        }
        pub fn mint_cleanup_recovery_handle(&self, phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>, cleanup: &crate::domain_computation::primary_graph::WorthQueryApplicationManagedCleanupReceipt) -> Result<WorthQueryRecoveryHandle, WorthQueryRecoveryHandleDenial> {
            self.inner.mint_cleanup_recovery_handle(phase, cleanup)
        }
        pub fn recovery_claim_status(&self, receipt: &WorthQueryApplicationCommitReceipt) -> Result<WorthQueryRecoveryClaimStatus, WorthQueryRecoveryHandleDenial> {
            self.inner.recovery_claim_status(receipt)
        }
    }
}

#[allow(unused_imports, dead_code)]
mod domain_computation_application_aftermath_recovery_handle_recorded_status {
    use worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryClaimStatus;
    use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
    use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;
    use crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding;
    use worth_query_installation::facade::ApplicationSchema;
    use crate::domain_computation::primary_graph::application_runtime::WorthQueryPrimaryGraphApplicationRuntime;
    use crate::domain_computation::primary_graph::product_operation::context::WorthQuerySelectedProductOperation;
    impl<Schema: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        pub fn resolve_admitted_recovery_claim_status<Operation, Input, Scope>(&self, phase: &WorthQueryAdvancementPhase<'_>, admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>, binding: WorthQueryApplicationIdempotencyBinding) -> Result<Option<WorthQueryRecoveryClaimStatus>, worth_query_execution_query::facade::domain_computation::application_aftermath::recovery_handle::WorthQueryRecoveryHandleDenial>
        where
            Input: Clone + Send + Sync + 'static, {
            self.inner.resolve_admitted_recovery_claim_status::<Operation, Input, Scope>(phase, admission, binding)
        }
    }
}
