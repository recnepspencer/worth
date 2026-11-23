use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::merge::data::{
    ExecutedMergeAspectDiagnosticRow, MaterializedAspectValue, MaterializedAspectValueEvidence,
};
use crate::transactions::data::primitives::RecordRef;

pub(super) fn executed_aspect_row_fields(
    row: &ExecutedMergeAspectDiagnosticRow,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "aspect_key",
            RelationalDiagnosticValue::AspectKey(row.aspect_key.clone()),
        ),
        (
            "class",
            RelationalDiagnosticValue::string(executed_aspect_class_label(row.class)),
        ),
        (
            "source_value",
            optional_materialized_aspect_value(row.source_value.as_ref()),
        ),
        (
            "target_value",
            optional_materialized_aspect_value(row.target_value.as_ref()),
        ),
        (
            "base_value",
            optional_materialized_aspect_value(row.base_value.as_ref()),
        ),
        (
            "shared_value",
            optional_materialized_aspect_value(row.shared_value.as_ref()),
        ),
        (
            "resolved_value",
            optional_materialized_aspect_value(row.resolved_value.as_ref()),
        ),
    ])
}

pub(super) fn record_ref_fields(record: &RecordRef) -> RelationalDiagnosticValue {
    match record {
        RecordRef::Entity(entity_id) => RelationalDiagnosticValue::EntityId(*entity_id),
        RecordRef::Relation(relation_id) => RelationalDiagnosticValue::RelationId(*relation_id),
    }
}

fn optional_materialized_aspect_value(
    value: Option<&MaterializedAspectValue>,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(value.map(materialized_aspect_value_fields))
}

fn materialized_aspect_value_fields(value: &MaterializedAspectValue) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "policy",
            RelationalDiagnosticValue::string(format!("{:?}", value.policy)),
        ),
        (
            "evidence",
            materialized_aspect_value_evidence(&value.evidence),
        ),
    ])
}

fn materialized_aspect_value_evidence(
    evidence: &MaterializedAspectValueEvidence,
) -> RelationalDiagnosticValue {
    match evidence {
        MaterializedAspectValueEvidence::EqualityWitnessDigest(digest) => {
            RelationalDiagnosticValue::object([
                (
                    "kind",
                    RelationalDiagnosticValue::string("equality_witness_digest"),
                ),
                ("digest", RelationalDiagnosticValue::string(digest.clone())),
            ])
        }
        MaterializedAspectValueEvidence::PinnedVisibleAspect {
            side,
            record,
            locator,
        } => RelationalDiagnosticValue::object([
            (
                "kind",
                RelationalDiagnosticValue::string("pinned_visible_aspect"),
            ),
            (
                "side",
                RelationalDiagnosticValue::string(format!("{side:?}")),
            ),
            ("record", record_ref_fields(record)),
            (
                "locator",
                RelationalDiagnosticValue::AspectValueLocator(locator.clone()),
            ),
        ]),
        MaterializedAspectValueEvidence::InlineAspectValue(value) => {
            RelationalDiagnosticValue::object([
                (
                    "kind",
                    RelationalDiagnosticValue::string("inline_aspect_value"),
                ),
                (
                    "value",
                    RelationalDiagnosticValue::AspectValue(value.clone()),
                ),
            ])
        }
    }
}

fn executed_aspect_class_label(
    class: crate::merge::data::ExecutedMergeAspectClass,
) -> &'static str {
    match class {
        crate::merge::data::ExecutedMergeAspectClass::AdoptSourceValue => "adopt_source_value",
        crate::merge::data::ExecutedMergeAspectClass::PreserveSharedValue => {
            "preserve_shared_value"
        }
        crate::merge::data::ExecutedMergeAspectClass::ReconcileValue => "reconcile_value",
    }
}
