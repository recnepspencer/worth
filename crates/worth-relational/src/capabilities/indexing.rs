use crate::identity::data::VersionId;
use crate::indexes::data::DerivedIndexGeneration;
use crate::replay::data::ReplaySnapshotSurface;
use crate::runtime::RelationalRuntime;

pub(crate) trait ReplayRead {
    fn replay_snapshot_surface_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: VersionId,
    ) -> Result<ReplaySnapshotSurface, crate::execution::RelationalExecutionDenialCause>;
    fn index_generations_at_version(&self, version_id: VersionId) -> Vec<DerivedIndexGeneration>;
}

impl ReplayRead for RelationalRuntime {
    fn replay_snapshot_surface_at_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        version_id: VersionId,
    ) -> Result<ReplaySnapshotSurface, crate::execution::RelationalExecutionDenialCause> {
        let projection = self.read_truth().project_historical_version(version_id);
        Ok(ReplaySnapshotSurface {
            version_id,
            entities: projection.all_authoritative_entity_records(execution)?,
            relations: projection.all_authoritative_relation_records(execution)?,
        })
    }

    fn index_generations_at_version(&self, version_id: VersionId) -> Vec<DerivedIndexGeneration> {
        self.index_access().generations_for_version(version_id)
    }
}
