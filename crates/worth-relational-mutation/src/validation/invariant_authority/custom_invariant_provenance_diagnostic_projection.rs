use crate::contracts::diagnostics::data::fields::RelationalDiagnosticValue;
use crate::contracts::validation::data::custom_rule::touched_scope::planned_records::{
    CustomInvariantTouchedSummary, StructuralCountView,
};
use crate::validation::custom_rule::{CustomInvariantProvenance, CustomInvariantTraversalSummary};

pub fn custom_invariant_provenance_diagnostic_value(
    provenance: &CustomInvariantProvenance,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "observation_kind",
            RelationalDiagnosticValue::string(provenance.observation_kind.diagnostic_label()),
        ),
        (
            "version_id",
            RelationalDiagnosticValue::VersionId(provenance.version_id),
        ),
        (
            "current_version_id",
            RelationalDiagnosticValue::VersionId(provenance.current_version_id),
        ),
        (
            "touched",
            touched_summary_diagnostic_value(&provenance.touched),
        ),
        ("counts", count_view_diagnostic_value(provenance.counts)),
        (
            "traversal",
            traversal_summary_diagnostic_value(provenance.traversal),
        ),
    ])
}

fn touched_summary_diagnostic_value(
    touched: &CustomInvariantTouchedSummary,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "visible_entity_ids",
            RelationalDiagnosticValue::array(
                touched
                    .visible_entity_ids
                    .iter()
                    .copied()
                    .map(RelationalDiagnosticValue::EntityId),
            ),
        ),
        (
            "visible_relation_ids",
            RelationalDiagnosticValue::array(
                touched
                    .visible_relation_ids
                    .iter()
                    .copied()
                    .map(RelationalDiagnosticValue::RelationId),
            ),
        ),
        (
            "touched_partition_ids",
            RelationalDiagnosticValue::array(
                touched
                    .touched_partition_ids
                    .iter()
                    .copied()
                    .map(RelationalDiagnosticValue::PartitionId),
            ),
        ),
        (
            "planned_entity_delete_count",
            RelationalDiagnosticValue::unsigned(touched.planned_entity_delete_count),
        ),
        (
            "planned_entity_create_count",
            RelationalDiagnosticValue::unsigned(touched.planned_entity_create_count),
        ),
        (
            "planned_relation_create_count",
            RelationalDiagnosticValue::unsigned(touched.planned_relation_create_count),
        ),
        (
            "planned_relation_delete_count",
            RelationalDiagnosticValue::unsigned(touched.planned_relation_delete_count),
        ),
        (
            "planned_relation_endpoint_update_count",
            RelationalDiagnosticValue::unsigned(touched.planned_relation_endpoint_update_count),
        ),
    ])
}

fn count_view_diagnostic_value(counts: StructuralCountView) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "visible_entity_count",
            RelationalDiagnosticValue::unsigned(counts.visible_entity_count()),
        ),
        (
            "visible_relation_count",
            RelationalDiagnosticValue::unsigned(counts.visible_relation_count()),
        ),
        (
            "planned_entity_create_count",
            RelationalDiagnosticValue::unsigned(counts.planned_entity_create_count()),
        ),
        (
            "planned_entity_delete_count",
            RelationalDiagnosticValue::unsigned(counts.planned_entity_delete_count()),
        ),
        (
            "planned_relation_create_count",
            RelationalDiagnosticValue::unsigned(counts.planned_relation_create_count()),
        ),
        (
            "planned_relation_delete_count",
            RelationalDiagnosticValue::unsigned(counts.planned_relation_delete_count()),
        ),
        (
            "planned_relation_endpoint_update_count",
            RelationalDiagnosticValue::unsigned(counts.planned_relation_endpoint_update_count()),
        ),
        (
            "touched_partition_count",
            RelationalDiagnosticValue::unsigned(counts.touched_partition_count()),
        ),
    ])
}

fn traversal_summary_diagnostic_value(
    traversal: CustomInvariantTraversalSummary,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "consumed_frontier",
            RelationalDiagnosticValue::unsigned(traversal.consumed_frontier),
        ),
        (
            "consumed_steps",
            RelationalDiagnosticValue::unsigned(traversal.consumed_steps),
        ),
        (
            "remaining_frontier",
            RelationalDiagnosticValue::unsigned(traversal.remaining_frontier),
        ),
        (
            "remaining_steps",
            RelationalDiagnosticValue::unsigned(traversal.remaining_steps),
        ),
        (
            "max_depth",
            RelationalDiagnosticValue::unsigned(traversal.max_depth),
        ),
    ])
}
