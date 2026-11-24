//! Freeze source DATA together, then leave both logical epochs before physical reads.
use super::super::checkpoint_components::CapturedCompleteComponents;
use super::{source::FrozenOriginalInputs, Denial};
use crate::branch::registry::CheckpointBranchMetadata;
use crate::durability::data::DurableRecordIdentityState;
use crate::runtime::state::subsystems::indexing::OriginalCheckpointIndexes;
use worth_execution::ExecutionRequest;

pub(super) mod freeze;

pub(super) struct FrozenCompleteMetadata {
    original: FrozenOriginalInputs,
    branches: CheckpointBranchMetadata,
    components: CapturedCompleteComponents,
    identity: DurableRecordIdentityState,
    indexes: crate::durability::derived_index_artifacts::FrozenIndexCheckpoint,
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
        let indexes = crate::durability::authority::checkpoint_image::capture_frozen_indexes(
            indexes, request,
        )?;
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
}
