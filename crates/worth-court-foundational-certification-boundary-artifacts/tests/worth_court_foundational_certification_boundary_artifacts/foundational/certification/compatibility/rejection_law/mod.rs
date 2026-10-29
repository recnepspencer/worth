use {serde_json::json};
use {worth_foundational::lower_json_record_aspect_state, worth_foundational::AspectContract, worth_foundational::ContractValidationDenial, worth_foundational::FieldKey, worth_foundational::JsonCompatibilityAspectInput, worth_foundational::JsonCompatibilityLoweringDenial, worth_foundational::ScalarAspectType};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::compatibility::json_lowering_fixtures::field_source_for, crate::certification::ordinary_context::compatibility::json_lowering_fixtures::scalar_input, crate::certification::ordinary_context::compatibility::json_lowering_fixtures::source_for, crate::certification::ordinary_context::compatibility::json_lowering_fixtures::task_summary_contract};
use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::scalar_contract};

#[test]
fn json_lowering_rejects_unknown_struct_fields_with_source_locus() {
    let outcome = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
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
        worth_proof::TransitionOutcome::Denied(worth_foundational::JsonCompatibilityLoweringDenial::UnknownStructField {
            source: field_source_for("task.summary", "extra"),
            field: worth_foundational::FieldKey::new("extra").expect("valid field key"),
        })
    );
}

#[test]
fn json_lowering_rejects_missing_required_fields_through_contract_validation() {
    let outcome = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
        task_summary_contract(),
        source_for("task.summary"),
        json!({ "title": "Ship it" }),
    )]);

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(worth_foundational::JsonCompatibilityLoweringDenial::ContractValidationDenied {
            source: field_source_for("task.summary", "done"),
            denial: worth_foundational::ContractValidationDenial::MissingRequiredField(field("done")),
        })
    );
}

#[test]
fn json_lowering_rejects_ambiguous_numeric_width() {
    let source = source_for("small");
    let outcome = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
        scalar_contract("small", 1, worth_foundational::ScalarAspectType::Int8),
        source.clone(),
        json!(999),
    )]);

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(worth_foundational::JsonCompatibilityLoweringDenial::AmbiguousNumericWidth {
            source,
            expected: worth_foundational::ScalarAspectType::Int8,
        })
    );
}

#[test]
fn json_lowering_rejects_zero_denominator_rational() {
    let source = source_for("ratio");
    let outcome = lower_json_record_aspect_state([scalar_input(
        "ratio",
        worth_foundational::ScalarAspectType::Rational,
        json!("3/0"),
    )]);

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(worth_foundational::JsonCompatibilityLoweringDenial::AmbiguousNumericWidth {
            source,
            expected: worth_foundational::ScalarAspectType::Rational,
        })
    );
}

#[test]
fn json_lowering_rejects_recursive_document_as_scalar_truth() {
    let source = source_for("name");
    let outcome = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
        scalar_contract("name", 1, worth_foundational::ScalarAspectType::String),
        source.clone(),
        json!({ "nested": "document" }),
    )]);

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::JsonCompatibilityLoweringDenial::UnsupportedRecursiveDocument {
                source,
                expected: worth_foundational::ScalarAspectType::String,
            }
        )
    );
}

#[test]
fn json_lowering_rejects_recursive_document_at_struct_field_locus() {
    let outcome = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
        task_summary_contract(),
        source_for("task.summary"),
        json!({
            "done": true,
            "title": { "nested": "document" }
        }),
    )]);

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::JsonCompatibilityLoweringDenial::UnsupportedRecursiveDocument {
                source: field_source_for("task.summary", "title"),
                expected: worth_foundational::ScalarAspectType::String,
            }
        )
    );
}

#[test]
fn json_lowering_rejects_incompatible_reference_shapes() {
    let contract =
        worth_foundational::AspectContract::reference_entity(key("entity.parent"), identity(41), revision(1));
    let source = source_for("entity.parent");

    let outcome = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
        contract,
        source.clone(),
        json!({
            "partition_id": 0,
            "local_slot": 9
        }),
    )]);

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(worth_foundational::JsonCompatibilityLoweringDenial::AmbiguousNumericWidth {
            source,
            expected: worth_foundational::ScalarAspectType::EntityRef,
        })
    );
}

#[test]
fn json_lowering_rejects_content_shape_with_shape_specific_denial() {
    let contract = worth_foundational::AspectContract::content_ref(key("blob.preview"), identity(52), revision(1));
    let source = source_for("blob.preview");

    let outcome = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
        contract,
        source.clone(),
        json!("inline-bytes-are-not-admitted"),
    )]);

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(worth_foundational::JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
            source,
            expected: "content reference id",
        })
    );
}

#[test]
fn json_lowering_rejects_scalar_content_ref_with_family_specific_shape_text() {
    let source = source_for("blob.slot");
    let outcome = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
        scalar_contract("blob.slot", 1, worth_foundational::ScalarAspectType::ContentRef),
        source.clone(),
        json!("not-a-content-id"),
    )]);

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(worth_foundational::JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
            source,
            expected: "content reference id",
        })
    );
}

#[test]
fn json_lowering_rejects_scalar_bytes_with_family_specific_shape_text() {
    let source = source_for("blob.bytes");
    let outcome = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
        scalar_contract("blob.bytes", 1, worth_foundational::ScalarAspectType::Bytes),
        source.clone(),
        json!("not-a-bytes-id"),
    )]);

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(worth_foundational::JsonCompatibilityLoweringDenial::JsonShapeNotAdmitted {
            source,
            expected: "bytes reference id",
        })
    );
}
