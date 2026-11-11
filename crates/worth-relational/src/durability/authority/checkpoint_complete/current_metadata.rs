//! One request-funded freeze of current owner state, independent of source class.
use super::{
    super::checkpoint_components::{capture_complete, CapturedCompleteComponents},
    Denial,
};
use crate::{
    branch::CheckpointBranchMetadata,
    durability::data::DurableRecordIdentityState,
    runtime::{FrozenCheckpointIndexes, RelationalRuntime},
};
use worth_execution::ExecutionRequest;

pub(super) struct FrozenCurrentCheckpointMetadata {
    pub(super) branches: CheckpointBranchMetadata,
    pub(super) components: CapturedCompleteComponents,
    pub(super) identity: DurableRecordIdentityState,
    pub(super) indexes: FrozenCheckpointIndexes,
    pub(super) symbol_frontier: u32,
    pub(super) main_cell: usize,
}
impl FrozenCurrentCheckpointMetadata {
    /// The caller holds configuration and a settled publication epoch.
    /// Every allocating snapshot enters its concrete owner's funded door.
    pub(super) fn capture(
        runtime: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
        runtime_name: &str,
        main: &crate::history::data::BranchId,
    ) -> Result<Self, Denial> {
        let branches = runtime
            .history
            .capture_checkpoint_branch_metadata(request, runtime_name)
            .map_err(|cause| match cause {
                crate::branch::CheckpointBranchMetadataCaptureDenial::Request(cause) => {
                    Denial::Request(cause)
                }
                crate::branch::CheckpointBranchMetadataCaptureDenial::SizeOverflow => {
                    Denial::Request(
                        crate::execution::RelationalExecutionDenialCause::MemoryOverflow,
                    )
                }
                crate::branch::CheckpointBranchMetadataCaptureDenial::SourceChanged => {
                    Denial::OriginalAuthorityUnavailable
                }
            })?;
        let main_cell = branches
            .branch_cells
            .iter()
            .position(|cell| &cell.branch_id == main)
            .ok_or(Denial::OriginalAuthorityUnavailable)?;
        let components = capture_complete(runtime, request, &branches)?;
        let identity = runtime
            .record_identity
            .checkpoint_identity_for_request(request)
            .map_err(crate::durability::checkpoint_history::capture::capture_stop)?;
        let indexes = runtime.indexes.checkpoint_inputs_for_request(request)?;
        let symbol_frontier = runtime
            .services
            .symbols
            .with_read(|symbols| symbols.identity_frontier());
        Ok(Self {
            branches,
            components,
            identity,
            indexes,
            symbol_frontier,
            main_cell,
        })
    }
}
