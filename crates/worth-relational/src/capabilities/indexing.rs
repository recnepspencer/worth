use crate::identity::data::VersionId;
use crate::replay::data::ReplaySnapshotSurface;
use crate::runtime::RelationalRuntime;

pub(crate) trait ReplayRead {
    fn replay_snapshot_surface_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: VersionId,
    ) -> Result<ReplaySnapshotSurface, crate::execution::RelationalExecutionDenialCause>;
}

impl ReplayRead for RelationalRuntime {
    fn replay_snapshot_surface_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: VersionId,
    ) -> Result<ReplaySnapshotSurface, crate::execution::RelationalExecutionDenialCause> {
        let view = self.read_truth().read_version(execution, version_id)?;
        Ok(ReplaySnapshotSurface {
            version_id,
            entities: view.entities().to_vec(),
            relations: view.relations().to_vec(),
        })
    }
}
