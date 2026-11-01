//! Immutable branch checkpoint DATA, shared without retaining live branch cells.
use crate::branch::RelationalBranchCellCheckpoint;
use crate::durability::data::DurableBranchLifecycleImage;
use crate::history::data::BranchId;
use std::sync::Arc;
use worth_execution::ExecutionMemoryReservation;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CheckpointBranchMetadata(Arc<CheckpointBranchMetadataFields>);

pub(crate) struct CheckpointBranchMetadataFields {
    pub(crate) runtime_name: String,
    pub(crate) branch_cells: Box<[RelationalBranchCellCheckpoint]>,
    pub(crate) branch_lifecycle: Box<[DurableBranchLifecycleImage]>,
    pub(crate) retired_branch_names: Box<[BranchId]>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl std::ops::Deref for CheckpointBranchMetadata {
    type Target = CheckpointBranchMetadataFields;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CheckpointBranchMetadata {
    pub(super) fn captured(
        runtime_name: String,
        branch_cells: Vec<RelationalBranchCellCheckpoint>,
        branch_lifecycle: Vec<DurableBranchLifecycleImage>,
        retired_branch_names: Vec<BranchId>,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self(Arc::new(CheckpointBranchMetadataFields {
            runtime_name,
            branch_cells: branch_cells.into_boxed_slice(),
            branch_lifecycle: branch_lifecycle.into_boxed_slice(),
            retired_branch_names: retired_branch_names.into_boxed_slice(),
            _memory: memory,
        }))
    }

    /// Legacy input construction is ordinary and carries no funding claim.
    pub(crate) fn from_ordinary_parts(
        runtime_name: String,
        branch_cells: Vec<RelationalBranchCellCheckpoint>,
        branch_lifecycle: Vec<DurableBranchLifecycleImage>,
        retired_branch_names: Vec<BranchId>,
    ) -> Self {
        Self::captured(
            runtime_name,
            branch_cells,
            branch_lifecycle,
            retired_branch_names,
            None,
        )
    }

    /// Public DurableCheckpoint keeps its existing raw movable value contract.
    /// This explicit legacy projection is not a controlled owning handoff.
    pub(crate) fn ordinary_owned_parts(
        &self,
    ) -> (
        String,
        Vec<RelationalBranchCellCheckpoint>,
        Vec<DurableBranchLifecycleImage>,
        Vec<BranchId>,
    ) {
        (
            self.runtime_name.clone(),
            self.branch_cells.to_vec(),
            self.branch_lifecycle.to_vec(),
            self.retired_branch_names.to_vec(),
        )
    }
}

pub(super) fn control_bytes() -> Option<usize> {
    Some(
        std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<CheckpointBranchMetadataFields>())
            .ok()?
            .0
            .pad_to_align()
            .size(),
    )
}

impl PartialEq for CheckpointBranchMetadataFields {
    fn eq(&self, other: &Self) -> bool {
        self.runtime_name == other.runtime_name
            && self.branch_cells == other.branch_cells
            && self.branch_lifecycle == other.branch_lifecycle
            && self.retired_branch_names == other.retired_branch_names
    }
}
impl Eq for CheckpointBranchMetadataFields {}
impl std::fmt::Debug for CheckpointBranchMetadataFields {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CheckpointBranchMetadata")
            .field("runtime_name", &self.runtime_name)
            .field("branch_cells", &self.branch_cells)
            .field("branch_lifecycle", &self.branch_lifecycle)
            .field("retired_branch_names", &self.retired_branch_names)
            .finish()
    }
}
