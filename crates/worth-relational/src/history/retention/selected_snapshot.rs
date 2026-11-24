//! Restricted reads backed by the exact selected root allocation.
use crate::branch::projection_view::snapshot_head::SelectedSnapshotHead;
use crate::branch::root::projection::SnapshotProjectionRoot;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{PartitionId, VersionId};
use crate::visibility::snapshot_states::VisibilitySnapshotStateKey;
use std::sync::Arc;

pub(crate) trait SelectedSnapshotSource:
    std::fmt::Debug + Send + Sync + std::any::Any
{
    fn project(&self) -> SnapshotProjectionRoot<'_>;
    fn project_head(&self, partition: PartitionId) -> SelectedSnapshotHead<'_>;
    fn root_id(&self) -> u64;
    fn commit_id(&self) -> Option<CommitId>;
}

/// Owner-selected immutable root retained by a snapshot handle binding.
#[derive(Clone, Debug)]
pub(crate) struct VisibilitySnapshotBasis {
    pub(super) key: VisibilitySnapshotStateKey,
    pub(super) root: Arc<dyn SelectedSnapshotSource>,
    pub(super) _retained_observation: Option<crate::mvcc::observation::RelationalBranchObservation>,
}

/// Exact root custody restricted to the selected snapshot's Head regions.
#[derive(Debug)]
pub(crate) struct SnapshotHeadSelection(Arc<dyn SelectedSnapshotSource>);

impl SnapshotHeadSelection {
    pub(crate) fn region(&self, partition: PartitionId) -> SelectedSnapshotHead<'_> {
        self.0.project_head(partition)
    }
}

impl VisibilitySnapshotBasis {
    pub(crate) fn key(&self) -> &VisibilitySnapshotStateKey {
        &self.key
    }

    pub(crate) fn branch_id(&self) -> &BranchId {
        self.key.branch_id()
    }

    pub(crate) const fn version_id(&self) -> VersionId {
        self.key.version_id()
    }

    pub(crate) fn head_selection(&self) -> SnapshotHeadSelection {
        SnapshotHeadSelection(self.root.clone())
    }

    pub(crate) fn projection_root(&self) -> SnapshotProjectionRoot<'_> {
        self.root.project()
    }

    pub(crate) fn take_observation_retirement(
        &mut self,
    ) -> Option<crate::mvcc::observation::RelationalObservationRetirement> {
        self._retained_observation
            .take()
            .map(|value| value.into_retirement())
    }

    pub(crate) fn root_id(&self) -> u64 {
        self.root.root_id()
    }

    pub(crate) fn commit_id(&self) -> Option<CommitId> {
        self.root.commit_id()
    }
}
