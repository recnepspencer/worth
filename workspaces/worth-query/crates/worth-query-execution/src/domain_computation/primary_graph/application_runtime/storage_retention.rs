//! Host maintenance borrows the exact application's original advancement request.
use super::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementPhase, WorthQueryForeignAdvancementPhase,
};
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::runtime::{
    RelationalRetentionPassProgress, RelationalRetentionPassStop,
};

/// Actual completed prefix. Maintenance can stop after reclaiming some records.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorthQueryStorageRetentionProgress {
    pub entity_reclaimable: usize,
    pub entity_reclaimed: usize,
    pub entity_chunks_scanned: usize,
    pub relation_reclaimable: usize,
    pub relation_reclaimed: usize,
    pub relation_chunks_scanned: usize,
    pub slots_scanned: usize,
    pub states_refreshed: usize,
}

/// The actual phase, original execution or authoritative-storage refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryStorageRetentionStop {
    ForeignPhase(WorthQueryForeignAdvancementPhase),
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    /// A live authoritative storage edition is being read. Retry after that
    /// reader releases it; historical Product observations are independent.
    StorageObserved,
}

/// A stopped pass preserving its completed storage effects and typed cause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryStorageRetentionDenial {
    pub progress: WorthQueryStorageRetentionProgress,
    pub stop: WorthQueryStorageRetentionStop,
}

impl From<&RelationalRetentionPassProgress> for WorthQueryStorageRetentionProgress {
    fn from(progress: &RelationalRetentionPassProgress) -> Self {
        let outcome = progress.outcome();
        Self {
            entity_reclaimable: outcome.entity_reclaimable,
            entity_reclaimed: outcome.entity_reclaimed,
            entity_chunks_scanned: outcome.entity_chunks_scanned,
            relation_reclaimable: outcome.relation_reclaimable,
            relation_reclaimed: outcome.relation_reclaimed,
            relation_chunks_scanned: outcome.relation_chunks_scanned,
            slots_scanned: progress.slots_scanned(),
            states_refreshed: progress.states_refreshed(),
        }
    }
}

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
            .execution_request_for(&self.product_runtime)
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
