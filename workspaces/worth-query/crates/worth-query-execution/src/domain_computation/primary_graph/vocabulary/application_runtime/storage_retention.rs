//! Moved whole from `domain_computation::primary_graph::application_runtime::storage_retention`: the items below and every impl they own.
use crate::domain_computation::request_execution::WorthQueryForeignAdvancementPhase;
use worth_relational::facade::runtime::RelationalRetentionPassProgress;

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
