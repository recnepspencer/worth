//! Index maintenance reads sealed root-local truth without runtime or snapshot custody.
use crate::branch::RelationalBranchRoot;
use crate::identity::data::PartitionId;
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::runtime::VisibilityProjectionView;
use crate::storage::residency::RelationalRegionHeadView;
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct IndexRootView {
    root: Arc<RelationalBranchRoot>,
    version: crate::identity::data::VersionId,
}
impl IndexRootView {
    pub(super) fn new(root: Arc<RelationalBranchRoot>) -> Self {
        let version = root
            .axes()
            .map(|axes| crate::identity::data::VersionId(axes.storage_version))
            .unwrap_or(crate::identity::data::VersionId(0));
        Self { root, version }
    }
    pub(super) fn from_projection(view: &VisibilityProjectionView) -> Result<Self, Denial> {
        let root = view.selected_root().ok_or(Denial::SnapshotUnavailable)?;
        if root
            .axes()
            .is_some_and(|axes| axes.storage_version != view.version_id().0)
        {
            return Err(Denial::BeforeRootMismatch);
        }
        Ok(Self {
            root: Arc::clone(root),
            version: view.version_id(),
        })
    }
    pub(super) fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.root
    }
    pub(super) fn version_id(&self) -> crate::identity::data::VersionId {
        self.version
    }
    pub(super) fn applied(
        root: Arc<RelationalBranchRoot>,
        version: crate::identity::data::VersionId,
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
                use crate::storage::residency::RelationalResidencyDemandFailure as Failure;
                Denial::Execution(match failure {
                    Failure::StorageUnavailable(cause) => {
                        crate::execution::RelationalExecutionDenialCause::StorageUnavailable(cause)
                    }
                    Failure::Request(cause) => cause,
                    Failure::Content(cause) => cause.into_execution_cause(),
                })
            })?;
        Ok(self.root.accepted_head_region_view(id))
    }
}
