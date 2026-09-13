use crate::history::{HistoryCatalogCounters, HistoryMetadataLedger};
/// Catalog-local snapshot. Byte values are World accounting charges, not
/// allocator residency or component-owner memory measurements.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeWorldHistorySnapshot {
    pub(crate) installed: usize,
    pub(crate) reserved: usize,
    pub(crate) metadata: HistoryMetadataLedger,
    pub(crate) costs: HistoryCatalogCounters,
}
impl RuntimeWorldHistorySnapshot {
    pub fn installed_commits(&self) -> usize {
        self.installed
    }
    pub fn reserved_commits(&self) -> usize {
        self.reserved
    }
    pub fn metadata(&self) -> HistoryMetadataLedger {
        self.metadata
    }
    pub fn costs(&self) -> HistoryCatalogCounters {
        self.costs
    }
}
