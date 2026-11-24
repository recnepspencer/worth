//! Index maintenance reads sealed root-local truth without runtime or snapshot custody.
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;
use crate::contracts::identity::data::primitives::PartitionId;
use crate::contracts::indexes::data::maintenance::DerivedIndexMaintenanceDenialKind as Denial;
use worth_relational_storage::internal_port::storage::residency::views::RelationalRegionHeadView;
use worth_relational_branch::internal_port::visibility::materialization::read_records::projection::view::VisibilityProjectionView;
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::root_snapshot_custody::VisibilityProjectionViewOwnerPort as _;

#[derive(Clone)]
pub struct IndexRootView {
    root: Arc<RelationalBranchRoot>,
    version: crate::contracts::identity::data::primitives::VersionId,
}
impl IndexRootView {
    pub(super) fn new(root: Arc<RelationalBranchRoot>) -> Self {
        let version = root
            .axes()
            .map(|axes| {
                crate::contracts::identity::data::primitives::VersionId(axes.storage_version)
            })
            .unwrap_or(crate::contracts::identity::data::primitives::VersionId(0));
        Self { root, version }
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
        })
    }
    pub(super) fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.root
    }
    pub(super) fn version_id(&self) -> crate::contracts::identity::data::primitives::VersionId {
        self.version
    }
    pub(super) fn applied(
        root: Arc<RelationalBranchRoot>,
        version: crate::contracts::identity::data::primitives::VersionId,
    ) -> Self {
        Self { root, version }
    }

    pub(super) fn head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
        work: &mut super::work::MaintenanceWork,
    ) -> Result<Option<RelationalRegionHeadView<'_>>, Denial> {
        work.prepare(33, 0)?;
        self.root
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
        Ok(self.root.accepted_head_region_view(id))
    }
}
