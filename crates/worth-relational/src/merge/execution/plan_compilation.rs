use std::collections::BTreeMap;
use std::sync::Arc;

use crate::merge::data::{
    BoundExecutableMergePlan, BoundExecutableMergeRecordPlan, ExecutionReadyLoweredMergePlan,
    MergeExecutionAuthorityBinding, MergeExecutionCompilationError,
    NormalizedRelationalMergeRequest, RuntimeInstanceId,
};
use crate::transactions::data::RecordRef;

use super::record_plan_compilation::compile_record_plan;

pub(super) fn compile_bound_executable_plan(
    runtime: &crate::runtime::RelationalRuntime,
    request: &NormalizedRelationalMergeRequest,
    execution_ready: &ExecutionReadyLoweredMergePlan,
) -> Result<BoundExecutableMergePlan, MergeExecutionCompilationError> {
    let parent_order = crate::merge::data::bound_parent_order(execution_ready);
    let source_records_by_ref = execution_ready
        .source_records
        .iter()
        .map(|record| (record.record_ref.clone(), record))
        .collect::<BTreeMap<_, _>>();

    let record_plans = execution_ready
        .lowered_records
        .iter()
        .map(|lowered_record| compile_record_plan(runtime, &source_records_by_ref, lowered_record))
        .collect::<Result<Vec<_>, _>>()?;
    let record_plans: Arc<[BoundExecutableMergeRecordPlan]> = Arc::from(record_plans);
    let diagnostics_plan =
        crate::merge::data::diagnostics_plan_from_record_plans(record_plans.as_ref());
    let executable_plan_digest = crate::merge::data::compiled_executable_plan_digest(
        request,
        parent_order.as_ref(),
        record_plans.as_ref(),
    );
    let (target_reference, target_truth_version) = runtime
        .history()
        .branch_reference_state(request.target_branch())
        .ok_or(
            MergeExecutionCompilationError::PreparedAuthorityBindingMismatch {
                detail: "target branch is unavailable during merge compilation",
            },
        )?;
    let (source_reference, source_truth_version) = runtime
        .history()
        .branch_reference_state(request.source_branch())
        .ok_or(
            MergeExecutionCompilationError::PreparedAuthorityBindingMismatch {
                detail: "source branch is unavailable during merge compilation",
            },
        )?;

    let binding = MergeExecutionAuthorityBinding {
        request: request.clone(),
        runtime_instance_id: RuntimeInstanceId(runtime.runtime_instance_id()),
        target_head_commit_id: execution_ready.basis.target_head.commit_id,
        source_head_commit_id: execution_ready.basis.source_head.commit_id,
        target_reference,
        source_reference,
        target_truth_version,
        source_truth_version,
        merge_base_commit_id: execution_ready.basis.merge_base.commit.commit_id,
        schema_snapshot_digest: crate::merge::data::schema_snapshot_digest(
            &execution_ready.schema_snapshot,
        ),
        freshness_policy: execution_ready.freshness_policy,
        executable_plan_digest,
    };

    Ok(BoundExecutableMergePlan {
        authority_binding: binding,
        parent_order,
        record_plans,
        diagnostics_plan,
    })
}

pub(super) type SourceRecordsByRef<'a> =
    BTreeMap<RecordRef, &'a crate::merge::data::VisibleMergeRecord>;
