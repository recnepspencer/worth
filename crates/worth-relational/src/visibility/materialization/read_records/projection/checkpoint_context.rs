//! Minimal owned read context for exact checkpoint-image verification.
use crate::storage::substrate::maintenance::ReconstructionLane;
use crate::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use worth_execution::ExecutionMemoryReservation;

/// Existing projection read handles and scalar observation metadata only.
/// This owns no runtime, configuration lock, mutation port or reconstruction grant.
#[derive(Debug)]
pub(crate) struct CheckpointProjectionContext {
    runtime_identity: u64,
    current_version: crate::identity::data::VersionId,
    instrumentation:
        crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
    symbols: crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
}
pub(crate) struct RequestOwnedCheckpointBasis {
    pub(crate) basis: crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    pub(crate) memory: Option<ExecutionMemoryReservation>,
}

pub(crate) struct RequestOwnedCheckpointProjection {
    view: VisibilityProjectionView,
    _memory: Option<ExecutionMemoryReservation>,
}
impl RequestOwnedCheckpointProjection {
    pub(crate) fn view(&self) -> &VisibilityProjectionView {
        &self.view
    }
}
impl CheckpointProjectionContext {
    pub(crate) fn from_read_handles(
        _lane: &ReconstructionLane<'_>,
        runtime_identity: u64,
        current_version: crate::identity::data::VersionId,
        instrumentation: crate::runtime::state::subsystems::services::instrumentation::RuntimeInstrumentation,
        symbols: crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
    ) -> Self {
        Self {
            runtime_identity,
            current_version,
            instrumentation,
            symbols,
        }
    }
    pub(crate) fn symbols(
        &self,
    ) -> crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable {
        self.symbols.clone()
    }
    pub(crate) fn project(
        &self,
        _lane: &ReconstructionLane<'_>,
        selection: RequestOwnedCheckpointBasis,
    ) -> RequestOwnedCheckpointProjection {
        RequestOwnedCheckpointProjection {
            view: VisibilityProjectionView {
                runtime_identity: self.runtime_identity,
                current_version: self.current_version,
                instrumentation: self.instrumentation.clone(),
                symbols: self.symbols.clone(),
                empty_history_schema: None,
                resident: None,
                basis: crate::visibility::snapshot_states::SnapshotStateBasis::Exact(
                    selection.basis,
                ),
            },
            _memory: selection.memory,
        }
    }
}
