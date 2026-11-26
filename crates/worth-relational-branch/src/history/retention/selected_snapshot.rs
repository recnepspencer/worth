//! Restricted reads backed by the exact selected root allocation.
use crate::branch::projection_view::snapshot_head::SelectedSnapshotHead;
use crate::branch::root::projection::SnapshotProjectionRoot;
use crate::history::data::{BranchId, CommitId};
use crate::identity::data::{PartitionId, VersionId};
use crate::storage::residency::OperationResidency;
use crate::visibility::snapshot_states::VisibilitySnapshotStateKey;
use std::sync::Arc;

pub trait SelectedSnapshotSource: std::fmt::Debug + Send + Sync + std::any::Any {
    fn project<'a>(&'a self, residency: &'a OperationResidency) -> SnapshotProjectionRoot<'a>;
    fn project_head<'a>(
        &'a self,
        partition: PartitionId,
        residency: &'a OperationResidency,
    ) -> SelectedSnapshotHead<'a>;
    fn root_id(&self) -> u64;
    fn commit_id(&self) -> Option<CommitId>;
}

/// Owner-selected immutable root retained by a snapshot handle binding.
///
/// A binding keeps only root custody. Each read takes [`Self::reading`], whose
/// residency its clones share for that one operation and drop with it, so a
/// long-lived handle never pins resident regions (Store M4 D1).
#[derive(Clone, Debug)]
pub struct VisibilitySnapshotBasis {
    pub(super) key: VisibilitySnapshotStateKey,
    pub(super) root: Arc<dyn SelectedSnapshotSource>,
    pub(super) _retained_observation: Option<crate::mvcc::observation::RelationalBranchObservation>,
    pub(super) residency: Arc<OperationResidency>,
}

/// Exact root custody restricted to the selected snapshot's Head regions.
#[derive(Debug)]
pub struct SnapshotHeadSelection {
    root: Arc<dyn SelectedSnapshotSource>,
    residency: OperationResidency,
}

impl SnapshotHeadSelection {
    pub fn region(&self, partition: PartitionId) -> SelectedSnapshotHead<'_> {
        self.root.project_head(partition, &self.residency)
    }
}

impl VisibilitySnapshotBasis {
    pub fn key(&self) -> &VisibilitySnapshotStateKey {
        &self.key
    }

    pub fn branch_id(&self) -> &BranchId {
        self.key.branch_id()
    }

    pub const fn version_id(&self) -> VersionId {
        self.key.version_id()
    }

    /// The same selection with a fresh residency for one read operation.
    pub fn reading(&self) -> Self {
        Self {
            residency: Default::default(),
            ..self.clone()
        }
    }

    pub fn head_selection(&self) -> SnapshotHeadSelection {
        SnapshotHeadSelection {
            root: self.root.clone(),
            residency: OperationResidency::new(),
        }
    }

    /// This read operation's residency.
    pub fn residency(&self) -> &OperationResidency {
        &self.residency
    }

    pub fn projection_root(&self) -> SnapshotProjectionRoot<'_> {
        self.root.project(&self.residency)
    }

    pub fn take_observation_retirement(
        &mut self,
    ) -> Option<crate::mvcc::observation::RelationalObservationRetirement> {
        self._retained_observation
            .take()
            .map(|value| value.into_retirement())
    }

    pub fn root_id(&self) -> u64 {
        self.root.root_id()
    }

    pub fn commit_id(&self) -> Option<CommitId> {
        self.root.commit_id()
    }
}

/// A snapshot basis lends partitions admitted by its own read residency.
impl crate::storage::overlay::PartitionAccess for VisibilitySnapshotBasis {
    fn get_partition(&self, id: PartitionId) -> Option<&crate::storage::overlay::PartitionState> {
        match self.projection_root().probe_full_region(id) {
            crate::storage::residency::PartitionProbe::Resident(partition) => Some(partition),
            _ => None,
        }
    }
    fn probe_full_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionProbe<'_> {
        self.projection_root().probe_full_region(id)
    }
    fn demand_full_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.projection_root()
            .demand_full_region_cached(execution, id)
    }
}

impl crate::storage::overlay::CurrentPartitionAccess for VisibilitySnapshotBasis {
    fn get_current_partition(
        &self,
        id: PartitionId,
    ) -> Option<crate::storage::residency::RelationalRegionHeadView<'_>> {
        self.projection_root().accepted_head_region_view(id)
    }
    fn probe_head_partition(
        &self,
        id: PartitionId,
    ) -> crate::storage::residency::PartitionHeadProbe<'_> {
        self.projection_root().probe_head_region(id)
    }
    fn demand_head_partition(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
    ) -> Result<(), crate::storage::residency::RelationalResidencyDemandFailure> {
        self.projection_root()
            .demand_head_region_cached(execution, id)
    }
    fn partition_ids(&self) -> Vec<PartitionId> {
        self.projection_root().partition_ids_iter().collect()
    }
    fn partition_ids_iter(&self) -> Box<dyn Iterator<Item = PartitionId> + '_> {
        Box::new(self.projection_root().partition_ids_iter())
    }
    fn partition_ids_with_memory(
        &self,
        memory: &mut dyn crate::storage::overlay::PartitionIdMemory,
    ) -> Result<Vec<PartitionId>, crate::storage::overlay::PartitionIdsDenial> {
        let root = self.projection_root();
        crate::storage::overlay::collect_partition_ids(
            root.region_count(),
            root.partition_ids_iter(),
            memory,
        )
    }
}
