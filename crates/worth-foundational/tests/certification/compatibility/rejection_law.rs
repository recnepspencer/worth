use serde_json::json;
use worth_foundational::{
    lower_json_record_aspect_state, AspectContract, ContractValidationDenial, FieldKey,
    JsonCompatibilityAspectInput, JsonCompatibilityLoweringDenial, ScalarAspectType,
};
use worth_proof::TransitionOutcome;

use super::json_lowering_fixtures::{
    field_source_for, scalar_input, source_for, task_summary_contract,
};
use crate::certification::foundational_vocabulary::{
    field, identity, key, revision, scalar_contract,
};

#[test]
fn json_lowering_rejects_unknown_struct_fields_with_source_locus() {
    let outcome = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        task_summary_contract(),
        source_for("task.summary"),
        json!({
            "done": true,
            "title": "Ship it",
            "extra": "nope"
        }),
    )]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(JsonCompatibilityLoweringDenial::UnknownStructField {
            source: field_source_for("task.summary", "extra"),
            field: FieldKey::new("extra").expect("valid field key"),
        })
    );
}

#[test]
fn json_lowering_rejects_missing_required_fields_through_contract_validation() {
    let outcome = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        task_summary_contract(),
        source_for("task.summary"),
        json!({ "title": "Ship it" }),
    )]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(JsonCompatibilityLoweringDenial::ContractValidationDenied {
            source: field_source_for("task.summary", "done"),
            denial: ContractValidationDenial::MissingRequiredField(field("done")),
        })
    );
}

#[test]
fn json_lowering_rejects_ambiguous_numeric_width() {
    let source = source_for("small");
    let outcome = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        scalar_contract("small", 1, ScalarAspectType::Int8),
        source.clone(),
        json!(999),
    )]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(JsonCompatibilityLoweringDenial::AmbiguousNumericWidth {
            source,
            expected: ScalarAspectType::Int8,
        })
    );
}

#[test]
fn json_lowering_rejects_zero_denominator_rational() {
    let source = source_for("ratio");
    let outcome = lower_json_record_aspect_state([scalar_input(
        "ratio",
        ScalarAspectType::Rational,
        json!("3/0"),
    )]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(JsonCompatibilityLoweringDenial::AmbiguousNumericWidth {
            source,
            expected: ScalarAspectType::Rational,
        })
    );
}

#[test]
fn json_lowering_rejects_recursive_document_as_scalar_truth() {
    let source = source_for("name");
    let outcome = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        scalar_contract("name", 1, ScalarAspectType::String),
        source.clone(),
        json!({ "nested": "document" }),
    )]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(
            JsonCompatibilityLoweringDenial::UnsupportedRecursiveDocument {
                source,
                expected: ScalarAspectType::String,
            }
        )
    );
}

#[test]
fn json_lowering_rejects_recursive_document_at_struct_field_locus() {
    let outcome = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        task_summary_contract(),
        source_for("task.summary"),
        json!({
            "done": true,
            "title": { "nested": "document" }
        }),
    )]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(
            JsonCompatibilityLoweringDenial::UnsupportedRecursiveDocument {
                source: field_source_for("task.summary", "title"),
                expected: ScalarAspectType::String,
            }
        )
    );
}

#[test]
fn json_lowering_rejects_incompatible_reference_shapes() {
    let contract =
        AspectContract::reference_entity(key("entity.parent"), identity(41), revision(1));
    let source = source_for("entity.parent");

    let outcome = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        contract,
        source.clone(),
        json!({
            "partition_id": 0,
            "local_slot": 9
        }),
    )]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(JsonCompatibilityLoweringDenial::AmbiguousNumericWidth {
            source,
            expected: ScalarAspectType::EntityRef,
        })
    );
}

#[test]
fn json_lowering_rejects_content_shape_with_shape_specific_denial() {
    let contract = AspectContract::content_ref(key("blob.preview"), identity(52), revision(1));
    let source = source_for("blob.preview");

    let outcome = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        contract,
        source.clone(),
        json!("inline-bytes-are-not-admitted"),
    )]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
            source,
            expected: "content reference id",
        })
    );
}

#[test]
fn json_lowering_rejects_scalar_content_ref_with_family_specific_shape_text() {
    let source = source_for("blob.slot");
    let outcome = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        scalar_contract("blob.slot", 1, ScalarAspectType::ContentRef),
        source.clone(),
        json!("not-a-content-id"),
    )]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
            source,
            expected: "content reference id",
        })
    );
}

#[test]
fn json_lowering_rejects_scalar_bytes_with_family_specific_shape_text() {
    let source = source_for("blob.bytes");
    let outcome = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        scalar_contract("blob.bytes", 1, ScalarAspectType::Bytes),
        source.clone(),
        json!("not-a-bytes-id"),
    )]);

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
            source,
            expected: "bytes reference id",
        })
    );
}
