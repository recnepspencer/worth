use {serde_json::json};
use {worth_foundational::admit_authoritative_record_aspect_state, worth_foundational::compatibility, worth_foundational::lower_json_record_aspect_state, worth_foundational::validate_aspect_value, worth_foundational::AspectValue, worth_foundational::ContractValidatedAspectValueView, worth_foundational::JsonCompatibilityAspectInput, worth_foundational::ScalarAspectType, worth_foundational::StructAspectValue};
use {worth_proof::TransitionOutcome};

use {crate::certification::ordinary_context::compatibility::json_lowering_fixtures::scalar_input, crate::certification::ordinary_context::compatibility::json_lowering_fixtures::source_for, crate::certification::ordinary_context::compatibility::json_lowering_fixtures::task_summary_contract};
use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key};

#[test]
fn json_lowering_matches_native_authoritative_state_for_equivalent_struct_truth() {
    let contract = task_summary_contract();
    let json_state = lower_json_record_aspect_state([worth_foundational::JsonCompatibilityAspectInput::new(
        contract.clone(),
        source_for("task.summary"),
        json!({
            "done": true,
            "title": "Ship it",
            "note": "native parity"
        }),
    )]);

    let native_value = worth_foundational::StructAspectValue::new([
        (field("title"), worth_foundational::AspectValue::String("Ship it".into())),
        (field("done"), worth_foundational::AspectValue::Bool(true)),
        (field("note"), worth_foundational::AspectValue::String("native parity".into())),
    ])
    .expect("unique native fields");
    let worth_proof::TransitionOutcome::Success(native_artifact) =
        validate_aspect_value(&contract, native_value.into())
    else {
        panic!("expected native validation");
    };
    let worth_proof::TransitionOutcome::Success(native_state) =
        admit_authoritative_record_aspect_state([native_artifact])
    else {
        panic!("expected native state admission");
    };

    let worth_proof::TransitionOutcome::Success(json_state) = json_state else {
        panic!("expected JSON compatibility lowering");
    };
    assert_eq!(json_state.payload(), native_state.payload());
}

#[test]
fn json_lowering_admits_unambiguous_scalar_wrapper_shapes() {
    let decimal = scalar_input("decimal", worth_foundational::ScalarAspectType::Decimal, json!("12.50"));
    let date = scalar_input("date", worth_foundational::ScalarAspectType::Date, json!(19_000));
    let uuid = scalar_input(
        "uuid",
        worth_foundational::ScalarAspectType::Uuid,
        json!([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]),
    );
    let rational = scalar_input("ratio", worth_foundational::ScalarAspectType::Rational, json!("3/4"));

    let outcome = lower_json_record_aspect_state([decimal, date, uuid, rational]);

    let worth_proof::TransitionOutcome::Success(state) = outcome else {
        panic!("expected unambiguous scalar wrapper lowering");
    };
    assert!(state.payload().get(&key("decimal")).is_some());
    assert!(state.payload().get(&key("date")).is_some());
    assert!(state.payload().get(&key("uuid")).is_some());
    assert!(state.payload().get(&key("ratio")).is_some());
}

#[test]
fn json_lowering_keeps_bytes_and_content_ref_scalar_families_distinct() {
    let bytes_contract = worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::scalar_contract(
        "blob.bytes",
        1,
        worth_foundational::ScalarAspectType::Bytes,
    );
    let content_contract = worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::scalar_contract(
        "blob.content_ref",
        1,
        worth_foundational::ScalarAspectType::ContentRef,
    );

    let worth_proof::TransitionOutcome::Success(bytes_artifact) =
        compatibility()
            .json()
            .lower_value(&bytes_contract, source_for("blob.bytes"), &json!(9))
    else {
        panic!("expected bytes compatibility lowering");
    };
    let worth_proof::TransitionOutcome::Success(content_artifact) = compatibility().json().lower_value(
        &content_contract,
        source_for("blob.content_ref"),
        &json!(9),
    ) else {
        panic!("expected content reference compatibility lowering");
    };

    let worth_foundational::ContractValidatedAspectValueView::Scalar(bytes_value) = bytes_artifact.payload().view()
    else {
        panic!("bytes contract produced non-scalar artifact");
    };
    let worth_foundational::ContractValidatedAspectValueView::Scalar(content_value) = content_artifact.payload().view()
    else {
        panic!("content reference contract produced non-scalar artifact");
    };

    assert_eq!(bytes_value.value_family(), worth_foundational::ScalarAspectType::Bytes);
    assert_eq!(content_value.value_family(), worth_foundational::ScalarAspectType::ContentRef);
    assert_ne!(bytes_value, content_value);
}
