use super::aspect_value_diagnostic_terms::{
    aspect_value_diagnostic_value, struct_aspect_value_diagnostic_value,
};
use super::RelationalDiagnosticValue;
use crate::identity::data::PartitionId;

#[path = "terminal_projection/aspect_projection_terms.rs"]
mod aspect_projection_terms;
#[path = "terminal_projection/value.rs"]
mod value;

use aspect_projection_terms::{
    aspect_field_locator_terminal_projection, aspect_value_locator_terminal_projection,
    canonical_basis_terminal_projection, diagnostic_mask_locator_terminal_projection,
    diagnostic_mask_terminal_projection,
};
pub(crate) use value::TerminalDiagnosticProjectionValue;

pub(crate) fn project_diagnostic_value_for_terminal_projection(
    value: &RelationalDiagnosticValue,
) -> TerminalDiagnosticProjectionValue {
    match value {
        RelationalDiagnosticValue::Null => TerminalDiagnosticProjectionValue::Null,
        RelationalDiagnosticValue::Bool(value) => TerminalDiagnosticProjectionValue::Bool(*value),
        RelationalDiagnosticValue::Unsigned(value) => {
            TerminalDiagnosticProjectionValue::Unsigned(*value)
        }
        RelationalDiagnosticValue::Signed(value) => {
            TerminalDiagnosticProjectionValue::Signed(*value)
        }
        RelationalDiagnosticValue::String(value) => {
            TerminalDiagnosticProjectionValue::String(value.clone())
        }
        RelationalDiagnosticValue::CanonicalBytes(bytes) => {
            TerminalDiagnosticProjectionValue::String(hex_bytes(bytes))
        }
        RelationalDiagnosticValue::Array(values) => TerminalDiagnosticProjectionValue::Array(
            values
                .iter()
                .map(project_diagnostic_value_for_terminal_projection)
                .collect(),
        ),
        RelationalDiagnosticValue::Object(fields) => TerminalDiagnosticProjectionValue::Object(
            fields
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        project_diagnostic_value_for_terminal_projection(value),
                    )
                })
                .collect(),
        ),
        RelationalDiagnosticValue::AspectKey(value) => {
            TerminalDiagnosticProjectionValue::String(value.as_str().to_string())
        }
        RelationalDiagnosticValue::FieldKey(value) => {
            TerminalDiagnosticProjectionValue::String(value.as_str().to_string())
        }
        RelationalDiagnosticValue::FieldPath(fields) => TerminalDiagnosticProjectionValue::Array(
            fields
                .fields()
                .iter()
                .map(|field| TerminalDiagnosticProjectionValue::String(field.as_str().to_string()))
                .collect(),
        ),
        RelationalDiagnosticValue::AspectValue(value) => {
            project_diagnostic_value_for_terminal_projection(&aspect_value_diagnostic_value(value))
        }
        RelationalDiagnosticValue::AspectFieldLocator(locator) => {
            aspect_field_locator_terminal_projection(locator)
        }
        RelationalDiagnosticValue::AspectValueLocator(locator) => {
            aspect_value_locator_terminal_projection(locator)
        }
        RelationalDiagnosticValue::StructAspectValue(value) => {
            project_diagnostic_value_for_terminal_projection(&struct_aspect_value_diagnostic_value(
                value,
            ))
        }
        RelationalDiagnosticValue::DiagnosticMask(mask) => {
            diagnostic_mask_terminal_projection(mask)
        }
        RelationalDiagnosticValue::DiagnosticMaskLocator(locator) => {
            diagnostic_mask_locator_terminal_projection(locator)
        }
        RelationalDiagnosticValue::CanonicalBasis(basis) => {
            canonical_basis_terminal_projection(basis)
        }
        RelationalDiagnosticValue::PartitionId(value) => unsigned(value.as_u64()),
        RelationalDiagnosticValue::KindId(value) => unsigned(value.as_u64()),
        RelationalDiagnosticValue::VersionId(value) => unsigned(value.as_u64()),
        RelationalDiagnosticValue::LineageId(value) => unsigned(value.as_u64()),
        RelationalDiagnosticValue::CommitId(value) => unsigned(value.0),
        RelationalDiagnosticValue::BranchId(value) => {
            TerminalDiagnosticProjectionValue::String(value.0.clone())
        }
        RelationalDiagnosticValue::SnapshotId(value) => unsigned(value.0),
        RelationalDiagnosticValue::DerivedIndexId(value) => unsigned(value.0),
        RelationalDiagnosticValue::DerivedIndexGenerationId(value) => unsigned(value.0),
        RelationalDiagnosticValue::CorrespondenceCandidateId(value) => unsigned(*value),
        RelationalDiagnosticValue::PatchStreamPosition(value) => unsigned(value.0),
        RelationalDiagnosticValue::ReplaySchemaVersion(value) => unsigned(value.0 as u64),
        RelationalDiagnosticValue::SchemaId(value) => {
            TerminalDiagnosticProjectionValue::String(value.0.clone())
        }
        RelationalDiagnosticValue::SchemaVersionId(value) => unsigned(value.0 as u64),
        RelationalDiagnosticValue::ContractId(value) => {
            TerminalDiagnosticProjectionValue::String(value.as_str().to_string())
        }
        RelationalDiagnosticValue::SchemaBoundaryFingerprint(value) => {
            TerminalDiagnosticProjectionValue::String(format!("{value:?}"))
        }
        RelationalDiagnosticValue::DescriptorSemanticsVersion(value) => unsigned(value.0 as u64),
        RelationalDiagnosticValue::DescriptorCanonicalBasisVersion(value) => {
            unsigned(value.0 as u64)
        }
        RelationalDiagnosticValue::EntityId(value) => record_id_terminal_projection(
            "entity",
            value.partition_id,
            value.local_slot_value(),
            value.generation_value(),
        ),
        RelationalDiagnosticValue::RelationId(value) => record_id_terminal_projection(
            "relation",
            value.partition_id,
            value.local_slot_value(),
            value.generation_value(),
        ),
    }
}

fn record_id_terminal_projection(
    record_kind: &'static str,
    partition_id: PartitionId,
    local_slot: u64,
    generation: u32,
) -> TerminalDiagnosticProjectionValue {
    object([
        ("record_kind", string(record_kind)),
        ("partition_id", unsigned(partition_id.as_u64())),
        ("local_slot", unsigned(local_slot)),
        ("generation", unsigned(generation as u64)),
    ])
}

fn object(
    fields: impl IntoIterator<Item = (&'static str, TerminalDiagnosticProjectionValue)>,
) -> TerminalDiagnosticProjectionValue {
    TerminalDiagnosticProjectionValue::Object(
        fields
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect(),
    )
}

fn string(value: impl Into<String>) -> TerminalDiagnosticProjectionValue {
    TerminalDiagnosticProjectionValue::String(value.into())
}

fn unsigned(value: u64) -> TerminalDiagnosticProjectionValue {
    TerminalDiagnosticProjectionValue::Unsigned(value)
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}
