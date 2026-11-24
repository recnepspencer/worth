use std::collections::BTreeMap;
use std::sync::Arc;

use crate::contracts::transactions::data::primitives::RecordRef;
use crate::merge::data::{
    BoundExecutableMergePlan, BoundExecutableMergeRecordPlan, ExecutionReadyLoweredMergePlan,
    MergeExecutionAuthorityBinding, MergeExecutionCompilationError,
    NormalizedRelationalMergeRequest, RuntimeInstanceId,
};

use super::record_plan_compilation::compile_record_plan;

pub fn compile_merge_records(
    aspect_plans: &crate::contracts::schema::data::aspect_contracts::lowered_plan::AspectContractPlanCatalog,
    request: &NormalizedRelationalMergeRequest,
    execution_ready: &ExecutionReadyLoweredMergePlan,
) -> Result<CompiledMergeRecords, MergeExecutionCompilationError> {
    let parent_order = crate::merge::data::bound_parent_order(&execution_ready.basis);
    let source_records_by_ref = execution_ready
        .source_records
        .iter()
        .map(|record| (record.record_ref.clone(), record))
        .collect::<BTreeMap<_, _>>();

    let record_plans = execution_ready
        .lowered_records
        .iter()
        .map(|lowered_record| {
            compile_record_plan(aspect_plans, &source_records_by_ref, lowered_record)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let record_plans: Arc<[BoundExecutableMergeRecordPlan]> = Arc::from(record_plans);
    let diagnostics_plan =
        crate::merge::data::diagnostics_plan_from_record_plans(record_plans.as_ref());
    let executable_plan_digest = crate::merge::data::compiled_executable_plan_digest(
        request,
        parent_order.as_ref(),
        record_plans.as_ref(),
    );
    Ok(CompiledMergeRecords {
        parent_order,
        record_plans,
        diagnostics_plan,
        executable_plan_digest,
    })
}

pub struct CompiledMergeRecords {
    parent_order: std::sync::Arc<[crate::contracts::history::data::coordinates::CommitId]>,
    record_plans: Arc<[BoundExecutableMergeRecordPlan]>,
    diagnostics_plan: crate::merge::data::MergeExecutionDiagnosticsPlan,
    executable_plan_digest: String,
}

impl CompiledMergeRecords {
    pub fn bind(
        self,
        request: &NormalizedRelationalMergeRequest,
        execution_ready: &ExecutionReadyLoweredMergePlan,
        runtime_instance_id: RuntimeInstanceId,
        target: (
            crate::contracts::branch::reference::observation::RelationalBranchReferenceObservation,
            crate::contracts::branch::version::RelationalBranchVersion,
        ),
        source: (
            crate::contracts::branch::reference::observation::RelationalBranchReferenceObservation,
            crate::contracts::branch::version::RelationalBranchVersion,
        ),
    ) -> BoundExecutableMergePlan {
        let (target_reference, target_truth_version) = target;
        let (source_reference, source_truth_version) = source;
        let Self {
            parent_order,
            record_plans,
            diagnostics_plan,
            executable_plan_digest,
        } = self;

        let binding = MergeExecutionAuthorityBinding {
            request: request.clone(),
            runtime_instance_id,
            target_head_commit_id: execution_ready.basis.target_head().commit_id,
            source_head_commit_id: execution_ready.basis.source_head().commit_id,
            target_reference,
            source_reference,
            target_truth_version,
            source_truth_version,
            merge_base_commit_id: execution_ready.basis.merge_base().commit().commit_id,
            schema_snapshot_digest: crate::merge::data::schema_snapshot_digest(
                &execution_ready.schema_snapshot,
            ),
            freshness_policy: execution_ready.freshness_policy,
            executable_plan_digest,
        };

        BoundExecutableMergePlan {
            authority_binding: binding,
            parent_order,
            record_plans,
            diagnostics_plan,
        }
    }
}

pub(super) type SourceRecordsByRef<'a> =
    BTreeMap<RecordRef, &'a crate::merge::data::VisibleMergeRecord>;
