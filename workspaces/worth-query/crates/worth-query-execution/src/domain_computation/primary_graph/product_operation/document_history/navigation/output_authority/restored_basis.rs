//! One exact restored basis and its grant-free Native selection read.
use super::*;
use worth_relational::facade::{
    mvcc::RelationalBranchObservation,
    runtime::{
        RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial,
        RelationalRuntime,
    },
};

#[derive(Clone)]
pub(in crate::domain_computation::primary_graph) struct HistoryRestoredOutputBasis {
    observation: Arc<WorthQueryApplicationReadObservation>,
    owned: Option<Arc<OwnedOutputSelection>>,
}
pub(in crate::domain_computation::primary_graph) struct OwnedOutputSelection {
    selected: RelationalBranchObservation,
    native: std::sync::OnceLock<Arc<RelationalHistoricalSelectionRead>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl HistoryRestoredOutputBasis {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history::navigation) fn new(
        observation: Arc<WorthQueryApplicationReadObservation>,
        selected: Option<RelationalBranchObservation>,
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            observation,
            owned: selected.map(|selected| {
                Arc::new(OwnedOutputSelection {
                    selected,
                    native: std::sync::OnceLock::new(),
                    _memory: memory,
                })
            }),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn observation(
        &self,
    ) -> &Arc<WorthQueryApplicationReadObservation> {
        &self.observation
    }
    pub(super) fn selected(&self) -> Option<&RelationalBranchObservation> {
        self.owned.as_ref().map(|owned| &owned.selected)
    }
    pub(in crate::domain_computation::primary_graph) fn native(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        native: &RelationalRuntime,
    ) -> Result<&RelationalHistoricalSelectionRead, RelationalHistoricalSelectionReadDenial> {
        let owned = self
            .owned
            .as_ref()
            .ok_or(RelationalHistoricalSelectionReadDenial::ForeignOwner)?;
        if owned.native.get().is_none() {
            let selected = &owned.selected;
            let read = native.read_truth().read_historical_selection(
                execution,
                selected,
                &self
                    .observation
                    .product_observation()
                    .basis()
                    .relational_basis()
                    .observation(),
            )?;
            // No callbacks run in this owner. Racing readers can only construct
            // the same authenticated immutable selection; failed reads aren't cached.
            let _ = owned.native.set(Arc::new(read));
        }
        Ok(owned
            .native
            .get()
            .expect("a successful exact selection read is retained")
            .as_ref())
    }
}
