//! Index maintenance reads sealed root-local truth without runtime or snapshot custody.
use crate::branch::RelationalBranchRoot;
use crate::identity::data::PartitionId;
use crate::indexes::data::DerivedIndexMaintenanceDenialKind as Denial;
use crate::runtime::VisibilityProjectionView;
use crate::storage::residency::RelationalRegionHeadView;
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct IndexRootView(Arc<RelationalBranchRoot>);
impl IndexRootView {
    pub(super) fn new(root: Arc<RelationalBranchRoot>) -> Self {
        Self(root)
    }
    pub(super) fn from_projection(view: &VisibilityProjectionView) -> Result<Self, Denial> {
        let root = view.selected_root().ok_or(Denial::SnapshotUnavailable)?;
        if root
            .axes()
            .is_some_and(|axes| axes.storage_version != view.version_id().0)
        {
            return Err(Denial::BeforeRootMismatch);
        }
        Ok(Self(Arc::clone(root)))
    }
    pub(super) fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.0
    }
    pub(super) fn version_id(&self) -> crate::identity::data::VersionId {
        self.0
            .axes()
            .map(|axes| crate::identity::data::VersionId(axes.storage_version))
            .unwrap_or(crate::identity::data::VersionId(0))
    }
    pub(super) fn head(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        id: PartitionId,
        work: &mut super::work::MaintenanceWork,
    ) -> Result<Option<RelationalRegionHeadView<'_>>, Denial> {
        work.prepare(33, 0)?;
        self.0
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
        Ok(self.0.accepted_head_region_view(id))
    }
}
