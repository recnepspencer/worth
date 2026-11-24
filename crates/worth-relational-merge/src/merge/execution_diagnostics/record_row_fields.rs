use crate::contracts::diagnostics::data::fields::{
    RelationalDiagnosticFields, RelationalDiagnosticValue,
};
use crate::merge::data::ExecutedMergeRecordDiagnosticRow;

use super::aspect_value_fields::{executed_aspect_row_fields, record_ref_fields};
use super::policy_boundary_fields::merge_policy_proof_boundary_fields;

pub(super) fn executed_record_diagnostic_fields(
    row: &ExecutedMergeRecordDiagnosticRow,
) -> RelationalDiagnosticFields {
    RelationalDiagnosticValue::object([
        (
            "record_class",
            RelationalDiagnosticValue::string(executed_record_class_label(row.class)),
        ),
        (
            "source_record",
            optional_record_ref(row.source_record.as_ref()),
        ),
        (
            "target_record",
            optional_record_ref(row.target_record.as_ref()),
        ),
        ("record", optional_record_ref(row.record.as_ref())),
        (
            "classification",
            RelationalDiagnosticValue::string(format!("{:?}", row.provenance.classification)),
        ),
        (
            "causal_disposition",
            RelationalDiagnosticValue::string(format!("{:?}", row.provenance.causal_disposition)),
        ),
        (
            "equality_witness_digest",
            RelationalDiagnosticValue::optional(
                row.equality_witness.as_ref().map(|witness| {
                    RelationalDiagnosticValue::string(witness.witness_digest.clone())
                }),
            ),
        ),
        (
            "deletion_semantics",
            RelationalDiagnosticValue::optional(
                row.deletion_semantics
                    .map(|semantics| RelationalDiagnosticValue::string(format!("{semantics:?}"))),
            ),
        ),
        (
            "lineage_continuity",
            RelationalDiagnosticValue::optional(
                row.lineage_continuity
                    .map(|verdict| RelationalDiagnosticValue::string(format!("{verdict:?}"))),
            ),
        ),
        (
            "policy_proof_boundary",
            merge_policy_proof_boundary_fields(row.provenance.policy_proof_boundary),
        ),
        (
            "applied_policies",
            RelationalDiagnosticValue::array(
                row.provenance
                    .applied_policies
                    .iter()
                    .map(|policy| RelationalDiagnosticValue::string(format!("{policy:?}"))),
            ),
        ),
        (
            "aspect_rows",
            RelationalDiagnosticValue::array(
                row.aspect_rows.iter().map(executed_aspect_row_fields),
            ),
        ),
    ])
    .into()
}

fn optional_record_ref(
    record: Option<&crate::contracts::transactions::data::primitives::RecordRef>,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::optional(record.map(record_ref_fields))
}

fn executed_record_class_label(
    class: crate::merge::data::ExecutedMergeRecordClass,
) -> &'static str {
    match class {
        crate::merge::data::ExecutedMergeRecordClass::AdoptSource => "adopt_source",
        crate::merge::data::ExecutedMergeRecordClass::PreserveShared => "preserve_shared",
        crate::merge::data::ExecutedMergeRecordClass::Reconcile => "reconcile",
        crate::merge::data::ExecutedMergeRecordClass::ConvergeDeletedOnBothSides => {
            "converge_deleted_on_both_sides"
        }
    }
}
