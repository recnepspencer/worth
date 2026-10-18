use serde_json::json;
use worth_foundational::{
    aspect_state_digest_preparation_basis, lower_json_record_aspect_state, validate_aspect_value,
    AspectValue, CanonicalDigestPreparationEntry, JsonCompatibilityAspectInput, ScalarAspectType,
    StructAspectValue,
};
use worth_proof::TransitionOutcome;

use super::readiness_fixtures::{admitted_state, ready_state, task_summary_contract};
use crate::certification::foundational_vocabulary::{field, key, revision, validated_scalar};

#[test]
fn digest_preparation_basis_is_stable_across_native_insertion_order() {
    let count = validated_scalar("count", 1, ScalarAspectType::Int64, AspectValue::Int64(7));
    let name = validated_scalar(
        "name",
        2,
        ScalarAspectType::String,
        AspectValue::String("Ada".into()),
    );

    let left_ready = ready_state(admitted_state([count.clone(), name.clone()]));
    let right_ready = ready_state(admitted_state([name, count]));

    assert_eq!(
        aspect_state_digest_preparation_basis(&left_ready),
        aspect_state_digest_preparation_basis(&right_ready)
    );
}

#[test]
fn digest_preparation_basis_matches_compatibility_lowering_for_equivalent_truth() {
    let contract = task_summary_contract();
    let native_value = StructAspectValue::new([
        (field("title"), AspectValue::String("Ship it".into())),
        (field("done"), AspectValue::Bool(true)),
    ])
    .expect("unique native fields");
    let TransitionOutcome::Success(native_entry) =
        validate_aspect_value(&contract, native_value.into())
    else {
        panic!("expected native validation");
    };
    let native_ready = ready_state(admitted_state([native_entry]));

    let json_state = lower_json_record_aspect_state([JsonCompatibilityAspectInput::new(
        contract,
        worth_foundational::BoundarySourceLocator::aspect(worth_foundational::AspectLocator::new(
            worth_foundational::LocatorAuthority::SupportOnly,
            key("task.summary"),
        )),
        json!({ "done": true, "title": "Ship it" }),
    )]);
    let TransitionOutcome::Success(json_state) = json_state else {
        panic!("expected compatibility lowering");
    };
    let json_ready = ready_state(json_state);

    assert_eq!(
        aspect_state_digest_preparation_basis(&native_ready),
        aspect_state_digest_preparation_basis(&json_ready)
    );
}

#[test]
fn digest_preparation_keeps_equal_looking_value_variants_distinct() {
    let signed = validated_scalar("number", 1, ScalarAspectType::Int64, AspectValue::Int64(1));
    let unsigned = validated_scalar(
        "number",
        1,
        ScalarAspectType::UInt64,
        AspectValue::UInt64(1),
    );
    let signed_ready = ready_state(admitted_state([signed]));
    let unsigned_ready = ready_state(admitted_state([unsigned]));

    assert_ne!(
        aspect_state_digest_preparation_basis(&signed_ready),
        aspect_state_digest_preparation_basis(&unsigned_ready)
    );
}

#[test]
fn struct_field_digest_basis_uses_canonical_field_order() {
    let contract = task_summary_contract();
    let value = StructAspectValue::new([
        (field("title"), AspectValue::String("Ship it".into())),
        (field("done"), AspectValue::Bool(true)),
    ])
    .expect("unique fields");
    let TransitionOutcome::Success(entry) = validate_aspect_value(&contract, value.into()) else {
        panic!("expected validation");
    };
    let ready = ready_state(admitted_state([entry]));

    assert_eq!(
        aspect_state_digest_preparation_basis(&ready),
        &[
            CanonicalDigestPreparationEntry::StateAspect {
                key: key("task.summary"),
                revision: revision(1),
            },
            CanonicalDigestPreparationEntry::StateStructFieldValue {
                key: key("task.summary"),
                field: field("done"),
                value: AspectValue::Bool(true),
            },
            CanonicalDigestPreparationEntry::StateStructFieldValue {
                key: key("task.summary"),
                field: field("title"),
                value: AspectValue::String("Ship it".into()),
            },
        ]
    );
}
