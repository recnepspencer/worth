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
    recording_model: Option<worth_relational::facade::identity::EntityId>,
    original_required_outputs: bool,
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
            recording_model: None,
            original_required_outputs: false,
            owned: selected.map(|selected| {
                Arc::new(OwnedOutputSelection {
                    selected,
                    native: std::sync::OnceLock::new(),
                    _memory: memory,
                })
            }),
        }
    }
    pub(super) fn recording(
        observation: Arc<WorthQueryApplicationReadObservation>,
        model: worth_relational::facade::identity::EntityId,
        original_required_outputs: bool,
    ) -> Self {
        Self {
            observation,
            owned: None,
            recording_model: Some(model),
            original_required_outputs,
        }
    }
    pub(super) fn has_original_required_outputs(&self) -> bool {
        self.original_required_outputs
    }
    pub(super) fn recording_model(&self) -> Option<worth_relational::facade::identity::EntityId> {
        self.recording_model
    }
    pub(super) fn owns_domain(&self) -> bool {
        self.owned.is_some() || self.recording_model.is_some()
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
