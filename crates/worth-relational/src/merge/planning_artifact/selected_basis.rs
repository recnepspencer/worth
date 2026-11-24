use crate::merge::data::{
    BranchTouchedRecordDelta, LoweredMergePlan, MergePlanningArtifactCore, MergePlanningError,
    MergeSchemaSnapshotDigestBasis, VisibleMergeRecord,
};
use crate::runtime::RelationalRuntime;

use super::performance_counters::record_merge_planning_counters;

pub(in crate::merge) fn merge_schema_snapshot_for_execution_ready(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    target_version_id: crate::identity::data::VersionId,
    source_records: &[VisibleMergeRecord],
    target_touched_records: &[BranchTouchedRecordDelta],
) -> Result<MergeSchemaSnapshotDigestBasis, crate::merge::data::MergePlanningError> {
    let target_view = runtime
        .read_truth()
        .read_version(execution, target_version_id)
        .map_err(crate::merge::data::MergePlanningError::Execution)?;
    Ok(super::schema_snapshot::merge_schema_snapshot(
        &runtime.config.schema.registry,
        source_records,
        &target_view,
        target_touched_records,
    ))
}

pub(in crate::merge) fn materialize_planning_artifact(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    plan: LoweredMergePlan,
) -> Result<MergePlanningArtifactCore, MergePlanningError> {
    materialize_planning_observation(execution, runtime, plan)
        .map(super::super::RelationalMergePlanningObservation::into_artifact)
}

pub(in crate::merge) fn materialize_planning_observation(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    plan: LoweredMergePlan,
) -> Result<super::super::RelationalMergePlanningObservation, MergePlanningError> {
    let schema_snapshot = merge_schema_snapshot_for_execution_ready(
        execution,
        runtime,
        plan.basis.target_head().version_id,
        &plan.source_records,
        &plan.target_delta.touched_records,
    )?;
    record_merge_planning_counters(runtime, &plan, schema_snapshot.touched_kinds.len());
    Ok(super::materialization::materialize_planning_observation(
        plan,
        schema_snapshot,
    ))
}
