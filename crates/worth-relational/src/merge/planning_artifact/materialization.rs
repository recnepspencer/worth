use crate::capabilities::RuntimeConfigSource;
use crate::merge::data::{LoweredMergePlan, MergePlanningArtifactCore};
use crate::runtime::RelationalRuntime;

use super::super::schema_reconciliation_witness::retained_schema_reconciliation_witness;
use super::super::strategy_witness::retained_merge_strategy_witness;
use super::digest_basis::merge_artifact_digest_basis;
use super::execution_authority_contract::lowered_artifact_execution_authority_contract;
use super::performance_counters::record_merge_planning_counters;
use super::schema_snapshot::merge_schema_snapshot;
use super::summaries::{merge_ancestry_summary, merge_request_summary};

pub(crate) fn materialize_planning_artifact(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    plan: LoweredMergePlan,
) -> MergePlanningArtifactCore {
    let target_view = runtime
        .read_truth()
        .read_version(execution, plan.basis.target_head.version_id);
    let schema_snapshot = merge_schema_snapshot(
        &runtime.runtime_config().schema.registry,
        plan.source_records.as_ref(),
        &target_view,
        plan.target_delta.touched_records.as_ref(),
    );
    let execution_authority_contract = lowered_artifact_execution_authority_contract();
    let digest_basis = merge_artifact_digest_basis(
        &plan,
        schema_snapshot.clone(),
        execution_authority_contract.clone(),
    );
    let schema_reconciliation_witness =
        retained_schema_reconciliation_witness(&plan, &schema_snapshot);
    let strategy_witness = retained_merge_strategy_witness(&plan, &execution_authority_contract);
    let request_summary = merge_request_summary(&plan);
    let ancestry_summary = merge_ancestry_summary(&plan);

    record_merge_planning_counters(runtime, &plan, schema_snapshot.touched_kinds.len());

    MergePlanningArtifactCore {
        request: plan.request.clone(),
        branch_basis: plan.basis.clone(),
        schema_snapshot,
        schema_reconciliation_witness,
        strategy_witness,
        execution_authority_contract,
        ancestry: plan.ancestry.clone(),
        identity_discovery: plan.identity_summary.clone(),
        conflict_classification: plan.conflict_summary,
        causal_annotation: plan.causal_summary,
        policy_resolution: plan.policy_summary.clone(),
        lowered_plan: plan.lowered_summary,
        decision_log: plan.decision_log.clone(),
        digest_basis,
        decision_log_digest_basis: plan.decision_log_digest_basis.clone(),
        summary: crate::merge::data::MergePlanningSummary {
            request_summary,
            ancestry_summary,
        },
    }
}
