//! Classify actual captured components against the SAME owner-issued current cut.
use super::Denial;
use crate::branch::{
    CheckpointBranchMetadata, RelationalBranchBasisDescriptor, RelationalBranchBasisPosture,
    RelationalBranchCellCheckpoint, RelationalBranchLifecyclePosture, RelationalBranchRoot,
    RelationalBranchRootSchemaAuthority,
};
use crate::history::data::BranchId;
use std::sync::Arc;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

pub(super) struct BranchImages<'a> {
    current: ExecutionOrderedMap<
        &'a BranchId,
        (
            &'a RelationalBranchCellCheckpoint,
            RelationalBranchLifecyclePosture,
        ),
    >,
}
impl<'a> BranchImages<'a> {
    pub(super) fn capture(
        metadata: &'a CheckpointBranchMetadata,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut current = ExecutionOrderedMap::default();
        if metadata.branch_cells.len() != metadata.branch_lifecycle.len() {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        for (cell, lifecycle) in metadata.branch_cells.iter().zip(&metadata.branch_lifecycle) {
            if cell.branch_id != lifecycle.branch_id {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            current
                .insert_cloned_native_for_request(
                    &&cell.branch_id,
                    &(cell, lifecycle.posture),
                    *request,
                )
                .map_err(|cause| {
                    crate::durability::checkpoint_history::capture::capture_stop(
                        crate::durability::checkpoint_history::capture::map_stop(cause),
                    )
                })?;
        }
        Ok(Self { current })
    }
    pub(super) fn is_current_component(
        &self,
        descriptor: &RelationalBranchBasisDescriptor,
    ) -> bool {
        self.current
            .get(&descriptor.branch_id())
            .is_some_and(|(cell, lifecycle)| {
                let posture = match lifecycle {
                    RelationalBranchLifecyclePosture::Live => RelationalBranchBasisPosture::Live,
                    RelationalBranchLifecyclePosture::Archived => {
                        RelationalBranchBasisPosture::Archived
                    }
                    RelationalBranchLifecyclePosture::Deleting => {
                        RelationalBranchBasisPosture::Deleting
                    }
                };
                cell.observation.generation() == descriptor.reference().generation()
                    && cell.truth_version == descriptor.truth_version()
                    // Captured descriptors use the canonical runtime frame;
                    // the SAME owner's cell still uses its actual runtime ID.
                    && match (cell.observation.target(), descriptor.reference().target()) {
                        (worth_foundational::FoundationalBranchTarget::Empty,
                         worth_foundational::FoundationalBranchTarget::Empty) => true,
                        (worth_foundational::FoundationalBranchTarget::Basis(actual),
                         worth_foundational::FoundationalBranchTarget::Basis(captured)) =>
                            actual.selected_commit_id() == captured.selected_commit_id()
                            && actual.version_id() == captured.version_id()
                            && actual.parent_commit_ids() == captured.parent_commit_ids()
                            && actual.roots() == captured.roots(),
                        (worth_foundational::FoundationalBranchTarget::Empty,
                         worth_foundational::FoundationalBranchTarget::Basis(_))
                        | (worth_foundational::FoundationalBranchTarget::Basis(_),
                           worth_foundational::FoundationalBranchTarget::Empty) => false,
                    }
                    && posture == descriptor.posture()
            })
    }
    pub(super) fn require_component(
        &self,
        descriptor: &RelationalBranchBasisDescriptor,
        root: &Arc<RelationalBranchRoot>,
    ) -> Result<Arc<RelationalBranchRootSchemaAuthority>, Denial> {
        let schema = root.retained_schema_authority();
        if schema.authority_digest() != descriptor.schema_commitment() {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        match descriptor.reference().target() {
            worth_foundational::FoundationalBranchTarget::Empty => {
                if root.descriptor().is_some()
                    || root.checkpoint_region_count() != 0
                    || descriptor.visibility_commitment() != [0; 32]
                {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
            }
            worth_foundational::FoundationalBranchTarget::Basis(target) => {
                if root.commit_id().map(|id| id.0) != Some(target.selected_commit_id())
                    || root.descriptor() != Some(target.roots())
                    || root.axes().is_none_or(|axes| {
                        axes.visibility.digest() != descriptor.visibility_commitment()
                    })
                {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
            }
        }
        Ok(schema)
    }
}
