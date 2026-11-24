//! The exact retained root and observation stay in their original allocation.
use super::snapshot_projection_root::SnapshotProjectionRoot;
use crate::branch::projection_view::snapshot_head::SelectedSnapshotHead;
use crate::branch::root::RelationalBranchRoot;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{PartitionId, VersionId};
use crate::visibility::snapshot_states::VisibilitySnapshotStateKey;
use std::sync::Arc;

/// Owner-selected immutable root retained by a snapshot handle binding.
#[derive(Clone, Debug)]
pub(crate) struct VisibilitySnapshotBasis {
    key: VisibilitySnapshotStateKey,
    pub(super) root: Arc<RelationalBranchRoot>,
    pub(super) _retained_observation: Option<crate::mvcc::observation::RelationalBranchObservation>,
}

/// Exact root custody restricted to the selected snapshot's Head regions.
#[derive(Debug)]
pub(crate) struct SnapshotHeadSelection(Arc<RelationalBranchRoot>);

impl SnapshotHeadSelection {
    pub(crate) fn region(&self, partition: PartitionId) -> SelectedSnapshotHead<'_> {
        self.0.project_selected_head(partition)
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
        SnapshotProjectionRoot::borrowed(&self.root)
    }

    pub(crate) fn take_observation_retirement(
        &mut self,
    ) -> Option<crate::mvcc::observation::RelationalObservationRetirement> {
        self._retained_observation
            .take()
            .map(|value| value.into_retirement())
    }

    pub(crate) fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.root
    }

    pub(crate) fn schema_registry(&self) -> &crate::schema::data::RelationalSchemaRegistry {
        self.root.schema_authority().registry()
    }

    pub(crate) fn entity_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::EntityId> + '_ {
        self.root.entity_ids_of_kind(kind, partition)
    }

    pub(crate) fn relation_ids_of_kind(
        &self,
        kind: crate::identity::data::KindId,
        partition: Option<PartitionId>,
    ) -> impl Iterator<Item = crate::identity::data::RelationId> + '_ {
        self.root.relation_ids_of_kind(kind, partition)
    }

    #[cfg(test)]
    pub(crate) fn partition_records(
        &self,
    ) -> &(impl crate::storage::overlay::CurrentPartitionAccess + crate::storage::overlay::PartitionAccess)
    {
        self.root.as_ref()
    }

    pub(crate) fn root_id(&self) -> u64 {
        self.root.id()
    }

    pub(crate) fn commit_id(&self) -> Option<CommitId> {
        self.root.commit_id()
    }
}
