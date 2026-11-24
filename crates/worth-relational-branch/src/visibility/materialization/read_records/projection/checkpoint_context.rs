//! Minimal owned read context for exact checkpoint-image verification.
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use worth_execution::ExecutionMemoryReservation;

/// Existing projection read handles and scalar observation metadata only.
/// This owns no runtime, configuration lock, mutation port or reconstruction grant.
#[derive(Debug)]
pub struct CheckpointProjectionContext {
    pub(super) runtime_identity: u64,
    pub(super) current_version: crate::identity::data::VersionId,
    pub(super) instrumentation:
        crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    pub(super) symbols:
        crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
}
pub struct RequestOwnedCheckpointProjection {
    pub(super) view: VisibilityProjectionView,
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}
impl RequestOwnedCheckpointProjection {
    pub fn view(&self) -> &VisibilityProjectionView {
        &self.view
    }
}
impl CheckpointProjectionContext {
    pub fn from_inputs(
        _lane: &ReconstructionLane<'_>,
        inputs: super::ProjectionInputs<'_>,
    ) -> Self {
        Self {
            runtime_identity: inputs.runtime_identity,
            current_version: inputs.history.current_version_id(),
            instrumentation: inputs.instrumentation.clone(),
            symbols: inputs.symbols.clone(),
        }
    }
    pub fn symbols(
        &self,
    ) -> crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable {
        self.symbols.clone()
    }
}
