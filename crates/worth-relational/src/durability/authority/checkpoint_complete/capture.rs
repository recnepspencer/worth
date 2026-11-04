//! Freeze source DATA together, then leave both logical epochs before physical reads.
use super::super::checkpoint_components::{capture_complete, CapturedCompleteComponents};
use super::{source::FrozenOriginalInputs, Denial};
use crate::branch::CheckpointBranchMetadata;
use crate::durability::data::DurableRecordIdentityState;
use crate::runtime::{FrozenCheckpointIndexes, OriginalCheckpointIndexes, RelationalRuntime};
use worth_execution::ExecutionRequest;

pub(super) struct FrozenCompleteMetadata {
    original: FrozenOriginalInputs,
    branches: CheckpointBranchMetadata,
    components: CapturedCompleteComponents,
    identity: DurableRecordIdentityState,
    indexes: FrozenCheckpointIndexes,
    symbol_frontier: u32,
    main_cell: usize,
}
pub(super) struct CapturedCompleteMetadata {
    pub(super) original: FrozenOriginalInputs,
    pub(super) branches: CheckpointBranchMetadata,
    pub(super) components: CapturedCompleteComponents,
    pub(super) identity: DurableRecordIdentityState,
    pub(super) indexes: OriginalCheckpointIndexes,
    pub(super) symbol_frontier: u32,
    pub(super) main_cell: usize,
}
impl FrozenCompleteMetadata {
    pub(super) fn resolve(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedCompleteMetadata, Denial> {
        let Self {
            mut original,
            branches,
            components,
            identity,
            indexes,
            symbol_frontier,
            main_cell,
        } = self;
        original.protect_original_read(request)?;
        let indexes = indexes.resolve_for_request(request)?;
        Ok(CapturedCompleteMetadata {
            original,
            branches,
            components,
            identity,
            indexes,
            symbol_frontier,
            main_cell,
        })
    }
    pub(super) fn capture(
        runtime: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let binding = runtime.configuration_binding();
        let configuration = binding.operation();
        let routes = runtime.history.canonical_checkpoint_gate();
        let snapshot = routes
            .with_checkpoint_selection(|selection| {
                let original = FrozenOriginalInputs::capture(runtime, &selection, request)?;
                let branches = runtime
                    .history
                    .capture_checkpoint_branch_metadata(
                        request,
                        &configuration.config.execution.runtime_name,
                    )
                    .map_err(branch_stop)?;
                let main_cell = branches
                    .branch_cells
                    .iter()
                    .position(|cell| cell.branch_id == configuration.config.history.main_branch)
                    .ok_or(Denial::OriginalAuthorityUnavailable)?;
                let components = capture_complete(runtime, request, &branches)?;
                let identity = runtime
                    .record_identity
                    .checkpoint_identity_for_request(request)
                    .map_err(stop)?;
                let indexes = runtime.indexes.checkpoint_inputs_for_request(request)?;
                let symbol_frontier = runtime
                    .services
                    .symbols
                    .with_read(|symbols| symbols.identity_frontier());
                Ok::<_, Denial>(Self {
                    original,
                    branches,
                    components,
                    identity,
                    indexes,
                    symbol_frontier,
                    main_cell,
                })
            })
            .map_err(super::super::checkpointing::checkpoint_admission_error)??;
        drop(configuration);
        // No protected provider callback, journal read or reconstruction was
        // performed while either of the original logical epochs was held.
        Ok(snapshot)
    }
}
fn branch_stop(cause: crate::branch::CheckpointBranchMetadataCaptureDenial) -> Denial {
    match cause {
        crate::branch::CheckpointBranchMetadataCaptureDenial::Request(cause) => {
            Denial::Request(cause)
        }
        crate::branch::CheckpointBranchMetadataCaptureDenial::SizeOverflow => {
            Denial::Request(crate::execution::RelationalExecutionDenialCause::MemoryOverflow)
        }
        crate::branch::CheckpointBranchMetadataCaptureDenial::SourceChanged => {
            Denial::OriginalAuthorityUnavailable
        }
    }
}
fn stop(cause: crate::durability::data::RelationalRecoveryStop) -> Denial {
    crate::durability::checkpoint_history::capture::capture_stop(cause)
}
