use crate::diagnostics::data::{RelationalDiagnosticFields, RelationalDiagnosticValue};
use crate::merge::data::{MergeExecutionDiagnosticsPlan, PreparedMergeExecution};
use crate::transactions::data::outcomes::plan_artifacts::{
    MergeExecutionStructuralSummary, MergeExecutionSummary,
};

pub(super) fn merge_execution_summary_fields(
    summary: &MergeExecutionSummary,
    structural_summary: &MergeExecutionStructuralSummary,
    commit_id: crate::history::data::CommitId,
) -> RelationalDiagnosticFields {
    diagnostic_fields([
        (
            "target_branch",
            RelationalDiagnosticValue::BranchId(summary.request.target_branch().clone()),
        ),
        (
            "source_branch",
            RelationalDiagnosticValue::BranchId(summary.request.source_branch().clone()),
        ),
        (
            "merge_intent",
            RelationalDiagnosticValue::string(format!("{:?}", summary.request.merge_intent())),
        ),
        (
            "target_head_commit_id",
            RelationalDiagnosticValue::CommitId(summary.target_head_commit_id),
        ),
        (
            "source_head_commit_id",
            RelationalDiagnosticValue::CommitId(summary.source_head_commit_id),
        ),
        (
            "merge_base_commit_id",
            RelationalDiagnosticValue::CommitId(summary.merge_base_commit_id),
        ),
        ("commit_id", RelationalDiagnosticValue::CommitId(commit_id)),
        (
            "executed_record_count",
            RelationalDiagnosticValue::unsigned(structural_summary.executed_record_count),
        ),
        (
            "adopted_source_record_count",
            RelationalDiagnosticValue::unsigned(structural_summary.adopted_source_record_count),
        ),
        (
            "preserved_shared_record_count",
            RelationalDiagnosticValue::unsigned(structural_summary.preserved_shared_record_count),
        ),
        (
            "reconciled_record_count",
            RelationalDiagnosticValue::unsigned(structural_summary.reconciled_record_count),
        ),
        (
            "converged_deleted_on_both_sides_count",
            RelationalDiagnosticValue::unsigned(
                structural_summary.converged_deleted_on_both_sides_count,
            ),
        ),
        (
            "deleted_on_both_sides_lineage_unchanged_count",
            RelationalDiagnosticValue::unsigned(
                structural_summary.deleted_on_both_sides_lineage_unchanged_count,
            ),
        ),
        (
            "emitted_mutation_intent_count",
            RelationalDiagnosticValue::unsigned(structural_summary.emitted_mutation_intent_count),
        ),
        (
            "emitted_entity_create_count",
            RelationalDiagnosticValue::unsigned(structural_summary.emitted_entity_create_count),
        ),
        (
            "emitted_relation_create_count",
            RelationalDiagnosticValue::unsigned(structural_summary.emitted_relation_create_count),
        ),
        (
            "emitted_entity_update_count",
            RelationalDiagnosticValue::unsigned(structural_summary.emitted_entity_update_count),
        ),
        (
            "execution_digest",
            RelationalDiagnosticValue::string(summary.execution_digest.clone()),
        ),
        (
            "diagnostics_digest",
            RelationalDiagnosticValue::string(summary.diagnostics_digest.clone()),
        ),
    ])
}

pub(super) fn merge_execution_artifact_header_fields(
    summary: &MergeExecutionSummary,
    plan: &MergeExecutionDiagnosticsPlan,
    commit_id: crate::history::data::CommitId,
) -> RelationalDiagnosticFields {
    diagnostic_fields([
        ("commit_id", RelationalDiagnosticValue::CommitId(commit_id)),
        (
            "target_branch",
            RelationalDiagnosticValue::BranchId(summary.request.target_branch().clone()),
        ),
        (
            "source_branch",
            RelationalDiagnosticValue::BranchId(summary.request.source_branch().clone()),
        ),
        (
            "execution_digest",
            RelationalDiagnosticValue::string(summary.execution_digest.clone()),
        ),
        (
            "diagnostics_digest",
            RelationalDiagnosticValue::string(plan.digest.clone()),
        ),
        (
            "executed_record_count",
            RelationalDiagnosticValue::unsigned(plan.executed_records.len()),
        ),
    ])
}

pub(super) fn merge_execution_failure_fields(
    prepared: &PreparedMergeExecution,
    error: &crate::merge::data::MergeExecutionError,
) -> RelationalDiagnosticFields {
    let binding = &prepared.bound_executable_plan().authority_binding;
    diagnostic_fields([
        (
            "target_branch",
            RelationalDiagnosticValue::BranchId(prepared.request().target_branch().clone()),
        ),
        (
            "source_branch",
            RelationalDiagnosticValue::BranchId(prepared.request().source_branch().clone()),
        ),
        (
            "merge_intent",
            RelationalDiagnosticValue::string(format!("{:?}", prepared.request().merge_intent())),
        ),
        (
            "target_head_commit_id",
            RelationalDiagnosticValue::CommitId(binding.target_head_commit_id),
        ),
        (
            "source_head_commit_id",
            RelationalDiagnosticValue::CommitId(binding.source_head_commit_id),
        ),
        (
            "merge_base_commit_id",
            RelationalDiagnosticValue::CommitId(binding.merge_base_commit_id),
        ),
        (
            "schema_snapshot_digest",
            RelationalDiagnosticValue::string(binding.schema_snapshot_digest.clone()),
        ),
        (
            "executable_plan_digest",
            RelationalDiagnosticValue::string(binding.executable_plan_digest.clone()),
        ),
        (
            "diagnostics_digest",
            RelationalDiagnosticValue::string(
                prepared
                    .bound_executable_plan()
                    .diagnostics_plan
                    .digest
                    .clone(),
            ),
        ),
        (
            "error",
            RelationalDiagnosticValue::string(format!("{error:?}")),
        ),
    ])
}

fn diagnostic_fields(
    entries: impl IntoIterator<Item = (&'static str, RelationalDiagnosticValue)>,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object(entries).into()
}
