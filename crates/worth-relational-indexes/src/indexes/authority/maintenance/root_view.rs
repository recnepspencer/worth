//! Index maintenance reads sealed root-local truth without runtime or snapshot custody.
use crate::contracts::identity::data::primitives::{PartitionId, VersionId};
use crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind as Denial;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::root::schema::RelationalBranchRootSchemaAuthority;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use worth_relational_branch::internal_port::branch::root_regions::partition_visitation::RelationalPartitionVisit;
use worth_relational_branch::internal_port::branch::root_snapshot_custody::VisibilityProjectionViewOwnerPort as _;
use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use worth_relational_storage::internal_port::storage::overlay::PartitionState;
use worth_relational_storage::internal_port::storage::residency::views::RelationalRegionHeadView;

pub(super) struct StagedSuccessorOverlay {
    pub(super) schema_authority: Arc<RelationalBranchRootSchemaAuthority>,
    pub(super) staged_partitions: BTreeMap<PartitionId, PartitionState>,
    pub(super) all_partitions: BTreeSet<PartitionId>,
    pub(super) entity_slot_count: usize,
    pub(super) relation_slot_count: usize,
    pub(super) region_count: usize,
}

/// One maintenance operation's view; its clones share one read residency.
#[derive(Clone)]
pub struct IndexRootView {
    root: Arc<RelationalBranchRoot>,
    version: VersionId,
    staged: Option<Arc<StagedSuccessorOverlay>>,
    residency: Arc<worth_relational_storage::internal_port::storage::residency::OperationResidency>,
}
impl IndexRootView {
    pub(super) fn new(root: Arc<RelationalBranchRoot>) -> Self {
        let version = root
            .axes()
            .map(|axes| VersionId(axes.storage_version))
            .unwrap_or(VersionId(0));
        Self {
            root,
            version,
            staged: None,
            residency: Default::default(),
        }
    }
    pub(super) fn staged_successor(
        predecessor_root: Arc<RelationalBranchRoot>,
        version: VersionId,
        overlay: StagedSuccessorOverlay,
    ) -> Self {
        Self {
            root: predecessor_root,
            version,
            staged: Some(Arc::new(overlay)),
            residency: Default::default(),
        }
    }
    pub fn from_projection(view: &VisibilityProjectionView) -> Result<Self, Denial> {
        let root = view
            .retain_selected_root()
            .ok_or(Denial::SnapshotUnavailable)?;
        if root
            .axes()
            .is_some_and(|axes| axes.storage_version != view.version_id().0)
        {
            return Err(Denial::BeforeRootMismatch);
        }
        Ok(Self {
            root,
            version: view.version_id(),
            staged: None,
            residency: Default::default(),
        })
    }
    pub(super) fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.root
    }
    pub(super) fn schema_authority(&self) -> &RelationalBranchRootSchemaAuthority {
        self.staged.as_ref().map_or_else(
            || self.root.schema_authority(),
            |staged| staged.schema_authority.as_ref(),
        )
    }
    pub(super) fn entity_slot_count(&self) -> usize {
        self.staged
            .as_ref()
            .map_or_else(|| self.root.entity_slot_count(), |s| s.entity_slot_count)
    }
    pub(super) fn relation_slot_count(&self) -> usize {
        self.staged.as_ref().map_or_else(
            || self.root.relation_slot_count(),
            |s| s.relation_slot_count,
        )
    }
    pub(super) fn region_count(&self) -> usize {
        self.staged
            .as_ref()
            .map_or_else(|| self.root.region_count(), |s| s.region_count)
    }
    pub(super) fn try_visit_partitions<E>(
        &self,
        mut visit: impl FnMut(RelationalPartitionVisit) -> Result<(), E>,
    ) -> Result<(), E> {
        if let Some(staged) = &self.staged {
            for &partition_id in &staged.all_partitions {
                visit(RelationalPartitionVisit::Partition(partition_id))?;
            }
            Ok(())
        } else {
            self.root.try_visit_partitions(visit)
        }
    }
    pub(super) fn version_id(&self) -> VersionId {
        self.version
    }
    pub(super) fn applied(root: Arc<RelationalBranchRoot>, version: VersionId) -> Self {
        Self {
            root,
            version,
            staged: None,
            residency: Default::default(),
        }
    }

    pub(super) fn head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
        work: &mut super::work::MaintenanceWork,
    ) -> Result<Option<RelationalRegionHeadView<'_>>, Denial> {
        work.prepare(33, 0)?;
        if let Some(staged) = &self.staged
            && let Some(partition) = staged.staged_partitions.get(&id)
        {
            return Ok(Some(RelationalRegionHeadView::from_owned_partition(
                partition,
            )));
        }
        self.root
            .read_within(&self.residency)
            .demand_region_head_cached(execution, id)
            .map_err(|failure| {
                use worth_relational_storage::internal_port::storage::residency::demand_failure::RelationalResidencyDemandFailure as Failure;
                Denial::Execution(match failure {
                    Failure::StorageUnavailable(cause) => {
                        crate::contracts::execution::denial_cause::RelationalExecutionDenialCause::StorageUnavailable(cause)
                    }
                    Failure::Request(cause) => cause,
                    Failure::Content(cause) => cause.into_execution_cause(),
                })
            })?;
        Ok(self
            .root
            .accepted_head_region_view_within(id, &self.residency))
    }
}
