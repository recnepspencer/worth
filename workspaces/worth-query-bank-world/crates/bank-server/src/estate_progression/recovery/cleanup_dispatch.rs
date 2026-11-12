//! First delivery and recovery consume actual native custody retained through cleanup.
use crate::{BankCommitRecoveryHandle, BankIdentityRuntime, BankRecoveryDenial};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    primary_graph::WorthQueryExternalEffectDispatch,
    primary_graph::{WorthQueryApplicationManagedCleanupReceipt, WorthQueryCleanupDispatchDenial},
};
impl BankIdentityRuntime {
    pub fn resume_original_cleanup_dispatch<'receipt>(
        &self,
        receipt: &'receipt mut WorthQueryApplicationManagedCleanupReceipt,
        scope: &WorthQueryRequestScope,
    ) -> Result<Option<&'receipt WorthQueryExternalEffectDispatch>, WorthQueryCleanupDispatchDenial>
    {
        self.with_application_advancement(scope, |phase| {
            self.application_runtime()
                .resume_cleanup_original_dispatch(&phase, receipt, scope)
        })
        .map_err(WorthQueryCleanupDispatchDenial::ExecutionDenied)?
    }
    pub fn open_cleanup_commit_recovery(
        &self,
        receipt: &WorthQueryApplicationManagedCleanupReceipt,
    ) -> Result<BankCommitRecoveryHandle, BankRecoveryDenial> {
        self.application_runtime().with_host_advancement(|phase| {
            self.application_runtime().mint_cleanup_recovery_handle(&phase, receipt)
                .map(|query| BankCommitRecoveryHandle { query }).map_err(BankRecoveryDenial::from_query)
        }).map_err(|cause| BankRecoveryDenial::from_query(worth_query_host::facade::primary_graph::WorthQueryRecoveryHandleDenial::new(worth_query_host::facade::primary_graph::WorthQueryRecoveryHandleDenialKind::CompletionExecutionRequest(cause))))?
    }
}
