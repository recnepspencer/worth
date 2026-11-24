//! Bind and recover the exact root behind a snapshot projection.
use super::selected_snapshot::{SelectedSnapshotSource, VisibilitySnapshotBasis};
use crate::branch::projection_view::snapshot_head::SelectedSnapshotHead;
use crate::branch::root::{projection::SnapshotProjectionRoot, RelationalBranchRoot};
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{PartitionId, VersionId};
use crate::visibility::snapshot_states::VisibilitySnapshotStateKey;
use std::sync::Arc;

impl SelectedSnapshotSource for RelationalBranchRoot {
    fn project(&self) -> SnapshotProjectionRoot<'_> {
        self.project_snapshot()
    }

    fn project_head(&self, partition: PartitionId) -> SelectedSnapshotHead<'_> {
        self.project_selected_head(partition)
    }

    fn root_id(&self) -> u64 {
        self.id()
    }

    fn commit_id(&self) -> Option<CommitId> {
        Self::commit_id(self)
    }
}

impl VisibilitySnapshotBasis {
    pub(crate) fn from_applied_root(
        branch_id: BranchId,
        version_id: VersionId,
        root: Arc<RelationalBranchRoot>,
    ) -> Self {
        Self {
            key: VisibilitySnapshotStateKey::for_selection(
                branch_id,
                version_id,
                root.id(),
                root.schema_authority().authority_digest(),
            ),
            root,
            _retained_observation: None,
        }
    }

    pub(crate) fn from_observation(
        observation: &crate::mvcc::observation::RelationalBranchObservation,
    ) -> Self {
        let version_id = observation.version_id();
        let root = Arc::clone(observation.selected_root());
        Self {
            key: VisibilitySnapshotStateKey::for_selection(
                observation.identity().branch_id().clone(),
                version_id,
                root.id(),
                root.schema_authority().authority_digest(),
            ),
            root,
            _retained_observation: Some(observation.clone()),
        }
    }

    pub(crate) fn root(&self) -> &RelationalBranchRoot {
        let original: &dyn std::any::Any = self.root.as_ref();
        original
            .downcast_ref::<RelationalBranchRoot>()
            .expect("the branch owner selected this root")
    }

    pub(crate) fn retain_root(&self) -> Arc<RelationalBranchRoot> {
        let original: Arc<dyn std::any::Any + Send + Sync> = self.root.clone();
        original
            .downcast::<RelationalBranchRoot>()
            .expect("the branch owner selected this root")
    }
}
