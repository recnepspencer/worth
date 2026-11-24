//! Build the projected read while preserving the admitted checkpoint root.
use super::checkpoint_context::{CheckpointProjectionContext, RequestOwnedCheckpointProjection};
use super::VisibilityProjectionView;
use crate::storage::substrate::maintenance::ReconstructionLane;
use worth_execution::ExecutionMemoryReservation;

pub(crate) struct RequestOwnedCheckpointBasis {
    pub(crate) basis: crate::visibility::snapshot_states::VisibilitySnapshotBasis,
    pub(crate) memory: Option<ExecutionMemoryReservation>,
}

impl CheckpointProjectionContext {
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
