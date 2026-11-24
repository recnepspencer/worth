use worth_foundational::facade::{
    AspectFieldLocator, AspectMask, AspectMaskLocator, AspectValueLocator, CanonicalBasisEntry,
    CanonicalBasisReadyArtifact, CanonicalFieldPath, DiagnosticMask,
};

use crate::canonical_basis_ready_sequence::canonical_basis_ready_sequence;

use super::value::TerminalDiagnosticProjectionValue;

pub(super) fn aspect_field_locator_terminal_projection(
    locator: &AspectFieldLocator,
) -> TerminalDiagnosticProjectionValue {
    object([
        ("locator_kind", string("aspect_field")),
        (
            "authority",
            string(format!("{:?}", locator.aspect().authority())),
        ),
        ("aspect_key", string(locator.aspect().aspect_key().as_str())),
        ("field_path", field_path_projection(locator.field_path())),
    ])
}

pub(super) fn aspect_value_locator_terminal_projection(
    locator: &AspectValueLocator,
) -> TerminalDiagnosticProjectionValue {
    match locator {
        AspectValueLocator::WholeAspect(aspect) => object([
            ("locator_kind", string("whole_aspect")),
            ("authority", string(format!("{:?}", aspect.authority()))),
            ("aspect_key", string(aspect.aspect_key().as_str())),
        ]),
        AspectValueLocator::StructField(field) => object([
            ("locator_kind", string("struct_field")),
            (
                "authority",
                string(format!("{:?}", field.aspect().authority())),
            ),
            ("aspect_key", string(field.aspect().aspect_key().as_str())),
            ("field_path", field_path_projection(field.field_path())),
        ]),
    }
}

pub(super) fn diagnostic_mask_terminal_projection(
    mask: &AspectMask<DiagnosticMask>,
) -> TerminalDiagnosticProjectionValue {
    if mask.is_whole_aspect() {
        return object([("mask_kind", string("whole_aspect"))]);
    }

    object([
        ("mask_kind", string("fields")),
        ("field_paths", field_paths_projection(mask.paths())),
    ])
}

pub(super) fn diagnostic_mask_locator_terminal_projection(
    locator: &AspectMaskLocator<DiagnosticMask>,
) -> TerminalDiagnosticProjectionValue {
    object([
        ("locator_kind", string("diagnostic_mask")),
        ("authority", string(format!("{:?}", locator.authority()))),
        ("aspect_key", string(locator.aspect_key().as_str())),
        ("field_paths", field_paths_projection(locator.paths())),
    ])
}

pub(super) fn canonical_basis_terminal_projection(
    basis: &CanonicalBasisReadyArtifact,
) -> TerminalDiagnosticProjectionValue {
    let ready_sequence = canonical_basis_ready_sequence(basis);
    object([
        ("basis_kind", string("canonical_basis_ready")),
        ("domain", string(format!("{:?}", ready_sequence.domain()))),
        ("version", string(ready_sequence.version().as_str())),
        (
            "entry_count",
            unsigned(ready_sequence.entries().len() as u64),
        ),
        (
            "entries",
            TerminalDiagnosticProjectionValue::Array(
                ready_sequence
                    .entries()
                    .iter()
                    .map(canonical_basis_entry_terminal_projection)
                    .collect(),
            ),
        ),
    ])
}

fn canonical_basis_entry_terminal_projection(
    entry: &CanonicalBasisEntry,
) -> TerminalDiagnosticProjectionValue {
    object([
        ("domain", string(format!("{:?}", entry.domain()))),
        ("locus", string(format!("{:?}", entry.locus()))),
        ("kind", string(format!("{:?}", entry.kind()))),
        ("value", string(format!("{:?}", entry.value()))),
    ])
}

fn field_paths_projection(paths: &[CanonicalFieldPath]) -> TerminalDiagnosticProjectionValue {
    TerminalDiagnosticProjectionValue::Array(paths.iter().map(field_path_projection).collect())
}

fn field_path_projection(path: &CanonicalFieldPath) -> TerminalDiagnosticProjectionValue {
    TerminalDiagnosticProjectionValue::Array(
        path.fields()
            .iter()
            .map(|field| string(field.as_str()))
            .collect(),
    )
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
