//! Freeze source DATA together, then leave both logical epochs before physical reads.
use super::super::checkpoint_components::CapturedCompleteComponents;
use super::source::FrozenOriginalInputs;
use worth_relational_contracts::internal_port::durability::data::native_checkpoint_capture_denial::RelationalNativeCheckpointCaptureDenial as Denial;
use worth_relational_branch::internal_port::branch::registry::checkpoint_capture::data::CheckpointBranchMetadata;
use worth_relational_storage::internal_port::runtime::state::subsystems::record_identity::paged_base::FrozenRecordIdentity;
use worth_relational_indexes::internal_port::runtime::state::subsystems::indexing::checkpoint_inputs::OriginalCheckpointIndexes;
use worth_execution::ExecutionRequest;

pub struct FrozenCompleteMetadata {
    pub original: FrozenOriginalInputs,
    pub branches: CheckpointBranchMetadata,
    pub components: CapturedCompleteComponents,
    pub identity: FrozenRecordIdentity,
    pub lineage_naming:
        worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::FrozenLineageNaming,
    pub indexes: crate::durability::derived_index_artifacts::FrozenIndexCheckpoint,
    pub symbol_frontier: u32,
    pub main_cell: usize,
}
pub struct CapturedCompleteMetadata {
    pub(super) original: FrozenOriginalInputs,
    pub(super) branches: CheckpointBranchMetadata,
    pub(super) components: CapturedCompleteComponents,
    pub(super) identity: FrozenRecordIdentity,
    pub(super) lineage_naming:
        worth_relational_lineage::internal_port::runtime::state::subsystems::lineage::FrozenLineageNaming,
    pub(super) indexes: OriginalCheckpointIndexes,
    pub(super) symbol_frontier: u32,
    pub(super) main_cell: usize,
}
impl FrozenCompleteMetadata {
    pub fn resolve(
        self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<CapturedCompleteMetadata, Denial> {
        let Self {
            mut original,
            branches,
            components,
            identity,
            lineage_naming,
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
            lineage_naming,
            indexes,
            symbol_frontier,
            main_cell,
        })
    }
}
