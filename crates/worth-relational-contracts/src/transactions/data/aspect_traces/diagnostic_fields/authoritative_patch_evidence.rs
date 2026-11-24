use crate::diagnostics::data::RelationalDiagnosticValue;
use crate::publication::patch::data::{
    PublishedAuthoritativeFieldSet, PublishedAuthoritativePatch,
};

use worth_foundational::facade::{
    AspectKey, AspectValue, AspectValueLocator, FieldKey, StructAspectValue,
};

pub(super) fn authoritative_patch_evidence_value(
    locator: &AspectValueLocator,
    patch: &PublishedAuthoritativePatch,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "evidence_kind",
            RelationalDiagnosticValue::string("authoritative_patch"),
        ),
        (
            "locator",
            RelationalDiagnosticValue::AspectValueLocator(locator.clone()),
        ),
        ("patch", authoritative_patch_value(patch)),
    ])
}

fn authoritative_patch_value(patch: &PublishedAuthoritativePatch) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::array(
        patch
            .changed_aspects()
            .into_iter()
            .map(|aspect_key| authoritative_patch_aspect_value(patch, aspect_key)),
    )
}

fn authoritative_patch_aspect_value(
    patch: &PublishedAuthoritativePatch,
    aspect_key: AspectKey,
) -> RelationalDiagnosticValue {
    if let Some(value) = patch.scalar_set_for(&aspect_key) {
        return authoritative_patch_whole_scalar_set_value(aspect_key, value.clone());
    }
    if let Some(value) = patch.struct_set_for(&aspect_key) {
        return authoritative_patch_whole_struct_set_value(aspect_key, value.clone());
    }
    let field_sets = patch.field_sets_for(&aspect_key).collect::<Vec<_>>();
    let field_clears = patch.field_clears_for(&aspect_key).collect::<Vec<_>>();
    if !field_sets.is_empty() || !field_clears.is_empty() {
        return authoritative_patch_field_level_value(aspect_key.clone(), field_sets, field_clears);
    }
    if patch.whole_clear_for(&aspect_key) {
        return authoritative_patch_whole_clear_value(aspect_key);
    }
    RelationalDiagnosticValue::object([
        (
            "patch_kind",
            RelationalDiagnosticValue::string("empty_or_unclassified"),
        ),
        (
            "aspect_key",
            RelationalDiagnosticValue::AspectKey(aspect_key),
        ),
    ])
}

fn authoritative_patch_whole_scalar_set_value(
    aspect_key: AspectKey,
    value: AspectValue,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "patch_kind",
            RelationalDiagnosticValue::string("whole_scalar_set"),
        ),
        (
            "aspect_key",
            RelationalDiagnosticValue::AspectKey(aspect_key),
        ),
        ("value", RelationalDiagnosticValue::AspectValue(value)),
    ])
}

fn authoritative_patch_whole_struct_set_value(
    aspect_key: AspectKey,
    value: StructAspectValue,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "patch_kind",
            RelationalDiagnosticValue::string("whole_struct_set"),
        ),
        (
            "aspect_key",
            RelationalDiagnosticValue::AspectKey(aspect_key),
        ),
        ("value", RelationalDiagnosticValue::StructAspectValue(value)),
    ])
}

fn authoritative_patch_field_level_value(
    aspect_key: AspectKey,
    field_sets: Vec<&PublishedAuthoritativeFieldSet>,
    field_clears: Vec<&FieldKey>,
) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "patch_kind",
            RelationalDiagnosticValue::string("field_level"),
        ),
        (
            "aspect_key",
            RelationalDiagnosticValue::AspectKey(aspect_key),
        ),
        (
            "field_sets",
            RelationalDiagnosticValue::array(field_sets.into_iter().map(|field_set| {
                RelationalDiagnosticValue::object([
                    (
                        "field",
                        RelationalDiagnosticValue::FieldKey(field_set.field.clone()),
                    ),
                    (
                        "value",
                        RelationalDiagnosticValue::AspectValue(field_set.value.clone()),
                    ),
                ])
            })),
        ),
        (
            "field_clears",
            RelationalDiagnosticValue::array(
                field_clears
                    .into_iter()
                    .cloned()
                    .map(RelationalDiagnosticValue::FieldKey),
            ),
        ),
    ])
}

fn authoritative_patch_whole_clear_value(aspect_key: AspectKey) -> RelationalDiagnosticValue {
    RelationalDiagnosticValue::object([
        (
            "patch_kind",
            RelationalDiagnosticValue::string("whole_clear"),
        ),
        (
            "aspect_key",
            RelationalDiagnosticValue::AspectKey(aspect_key),
        ),
    ])
}
