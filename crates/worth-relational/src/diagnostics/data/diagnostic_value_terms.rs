use super::RelationalDiagnosticValue;

pub fn aspect_shape_diagnostic_value(
    shape: &worth_foundational::AspectShape,
) -> RelationalDiagnosticValue {
    match shape {
        worth_foundational::AspectShape::Scalar(scalar_type) => {
            RelationalDiagnosticValue::object([
                ("shape_kind", RelationalDiagnosticValue::string("scalar")),
                (
                    "scalar_type",
                    RelationalDiagnosticValue::string(format!("{scalar_type:?}")),
                ),
            ])
        }
        worth_foundational::AspectShape::Struct(struct_shape) => {
            RelationalDiagnosticValue::object([
                ("shape_kind", RelationalDiagnosticValue::string("struct")),
                (
                    "field_count",
                    RelationalDiagnosticValue::Unsigned(struct_shape.fields().len() as u64),
                ),
            ])
        }
        worth_foundational::AspectShape::Reference(reference_type) => {
            RelationalDiagnosticValue::object([
                ("shape_kind", RelationalDiagnosticValue::string("reference")),
                (
                    "reference_type",
                    RelationalDiagnosticValue::string(format!("{reference_type:?}")),
                ),
            ])
        }
        worth_foundational::AspectShape::Opaque(opaque_type) => {
            RelationalDiagnosticValue::object([
                ("shape_kind", RelationalDiagnosticValue::string("opaque")),
                (
                    "opaque_type",
                    RelationalDiagnosticValue::string(format!("{opaque_type:?}")),
                ),
            ])
        }
    }
}
