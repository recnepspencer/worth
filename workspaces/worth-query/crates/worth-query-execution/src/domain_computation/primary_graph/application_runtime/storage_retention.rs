//! Host maintenance borrows the exact application's original advancement request.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::{
    RelationalRetentionPassProgress, RelationalRetentionPassStop,
};
use {
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
    crate::domain_computation::request_execution::WorthQueryForeignAdvancementPhase,
};
use crate::domain_computation::primary_graph::vocabulary::application_runtime::storage_retention::WorthQueryStorageRetentionDenial;
use crate::domain_computation::primary_graph::vocabulary::application_runtime::storage_retention::WorthQueryStorageRetentionProgress;
use crate::domain_computation::primary_graph::vocabulary::application_runtime::storage_retention::WorthQueryStorageRetentionStop;





impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Try one storage retention pass under the installed policy and original
    /// advancement controls. A refusal includes all work already performed;
    /// retrying does not roll it back. No logical Product commit is created.
    /// Historical observations retain their existing visibility contract.
    ///
    /// Save this result outside the advancement callback: an original request
    /// stop also refuses the enclosing advancement's final handoff.
    pub fn try_reclaim_retired_storage(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryStorageRetentionProgress, WorthQueryStorageRetentionDenial> {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|cause| WorthQueryStorageRetentionDenial {
                progress: WorthQueryStorageRetentionProgress::default(),
                stop: WorthQueryStorageRetentionStop::ForeignPhase(cause),
            })?;
        let handle = self
            .runtime
            .primary_graph()
            .expect("installed primary graph")
            .integration_handle();
        handle.with_query_runtime_mut(&execution, |runtime, _| {
            runtime
                .retention()
                .try_run_pass(&execution)
                .map(|progress| WorthQueryStorageRetentionProgress::from(&progress))
                .map_err(|denial| WorthQueryStorageRetentionDenial {
                    progress: WorthQueryStorageRetentionProgress::from(denial.progress()),
                    stop: match denial.stop() {
                        RelationalRetentionPassStop::Request(cause) => {
                            WorthQueryStorageRetentionStop::Execution(cause)
                        }
                        RelationalRetentionPassStop::StorageObserved => {
                            WorthQueryStorageRetentionStop::StorageObserved
                        }
                    },
                })
        })
    }
}
