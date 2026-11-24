use crate::diagnostics::data::{RelationalDiagnosticFields, RelationalDiagnosticValue};
use crate::storage::data::RetentionPlan;

pub fn retention_plan_inspection_fields(
    plan: &RetentionPlan,
    branch_replay_overlap_entities: usize,
    branch_replay_overlap_relations: usize,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "retention_fence_version",
            RelationalDiagnosticValue::VersionId(plan.retention_fence_version),
        ),
        (
            "active_snapshot_count",
            RelationalDiagnosticValue::unsigned(plan.active_snapshot_count),
        ),
        (
            "branch_pinned_entities",
            RelationalDiagnosticValue::unsigned(plan.branch_pinned_entities),
        ),
        (
            "replay_pinned_entities",
            RelationalDiagnosticValue::unsigned(plan.replay_pinned_entities),
        ),
        (
            "snapshot_pinned_entities",
            RelationalDiagnosticValue::unsigned(plan.snapshot_pinned_entities),
        ),
        (
            "branch_pinned_relations",
            RelationalDiagnosticValue::unsigned(plan.branch_pinned_relations),
        ),
        (
            "replay_pinned_relations",
            RelationalDiagnosticValue::unsigned(plan.replay_pinned_relations),
        ),
        (
            "snapshot_pinned_relations",
            RelationalDiagnosticValue::unsigned(plan.snapshot_pinned_relations),
        ),
        (
            "branch_replay_overlap_entities",
            RelationalDiagnosticValue::unsigned(branch_replay_overlap_entities),
        ),
        (
            "branch_replay_overlap_relations",
            RelationalDiagnosticValue::unsigned(branch_replay_overlap_relations),
        ),
        (
            "reclaimable_entities",
            RelationalDiagnosticValue::unsigned(plan.reclaimable_entities),
        ),
        (
            "reclaimable_relations",
            RelationalDiagnosticValue::unsigned(plan.reclaimable_relations),
        ),
    ])
    .into()
}
