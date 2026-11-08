//! Freeze source DATA together, then leave both logical epochs before physical reads.
use super::super::checkpoint_components::CapturedCompleteComponents;
use super::current_metadata::FrozenCurrentCheckpointMetadata;
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
        placement: &crate::durability::data::RelationalCheckpointArtifactPolicy<'_>,
    ) -> Result<Self, Denial> {
        let binding = runtime.configuration_binding();
        let configuration = binding.operation();
        let routes = runtime.history.canonical_checkpoint_gate();
        let snapshot = routes
            .with_checkpoint_selection(|selection| {
                let original = FrozenOriginalInputs::capture(runtime, &selection, request)?;
                // Home selection retains immutable members by reference. Reject
                // an Embedded demand before protecting/reading any source body.
                if original.base.is_some() && placement.is_embedded() {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                let FrozenCurrentCheckpointMetadata {
                    branches,
                    components,
                    identity,
                    indexes,
                    symbol_frontier,
                    main_cell,
                } = FrozenCurrentCheckpointMetadata::capture(
                    runtime,
                    request,
                    &configuration.config.execution.runtime_name,
                    &configuration.config.history.main_branch,
                )?;
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
