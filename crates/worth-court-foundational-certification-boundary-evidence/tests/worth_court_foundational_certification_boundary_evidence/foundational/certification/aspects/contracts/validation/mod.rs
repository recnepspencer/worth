#[allow(unused_imports)]
use crate::certification::ordinary_context::aspects::contracts::validation::*;
use {worth_foundational::validate_aspect_value, worth_foundational::ContentRefId};

use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision,
};

#[test]
fn scalar_contract_validation_returns_proof_bearing_artifact() {
    let contract = worth_foundational::AspectContract::scalar(
        key("temperature.celsius"),
        identity(1),
        revision(3),
        worth_foundational::ScalarAspectType::Float64,
    );

    let outcome = validate_aspect_value(
        &contract,
        worth_foundational::AspectValue::Float64(worth_foundational::CanonicalF64::from_f64(21.0))
            .into(),
    );

    let worth_proof::TransitionOutcome::Success(artifact) = outcome else {
        panic!("expected validated scalar artifact");
    };

    match artifact.payload().view() {
        worth_foundational::ContractValidatedAspectValueView::Scalar(value) => {
            assert_eq!(artifact.payload().key().as_str(), "temperature.celsius");
            assert_eq!(
                value.value_family(),
                worth_foundational::ScalarAspectType::Float64
            );
            assert_eq!(artifact.payload().contract_revision(), revision(3));
        }
        worth_foundational::ContractValidatedAspectValueView::Struct(_) => {
            panic!("scalar contract produced struct")
        }
    }
}

#[test]
fn scalar_contract_validation_denies_wrong_width() {
    let contract = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(1),
        revision(1),
        worth_foundational::ScalarAspectType::Int64,
    );

    let outcome =
        validate_aspect_value(&contract, worth_foundational::AspectValue::Int32(9).into());

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::ContractValidationDenial::ScalarTypeMismatch {
                expected: worth_foundational::ScalarAspectType::Int64,
                found: worth_foundational::ScalarAspectType::Int32,
            }
        )
    );
}

#[test]
fn struct_contract_validation_denies_missing_required_field() {
    let contract = task_summary_contract();

    let outcome = validate_aspect_value(
        &contract,
        worth_foundational::StructAspectValue::new([(
            worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field("title"),
            worth_foundational::AspectValue::String("Ship it".into()),
        )])
        .expect("unique fields")
        .into(),
    );

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(worth_foundational::ContractValidationDenial::MissingRequiredField(
            worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field("done")
        ))
    );
}

#[test]
fn struct_contract_validation_denies_field_type_mismatch() {
    let contract = task_summary_contract();

    let outcome = validate_aspect_value(
        &contract,
        worth_foundational::StructAspectValue::new([
            (
                worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field("title"),
                worth_foundational::AspectValue::String("Ship it".into()),
            ),
            (
                worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field("done"),
                worth_foundational::AspectValue::String("no".into()),
            ),
        ])
        .expect("unique fields")
        .into(),
    );

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(worth_foundational::ContractValidationDenial::FieldTypeMismatch {
            field: worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field("done"),
            expected: worth_foundational::ScalarAspectType::Bool,
            found: worth_foundational::ScalarAspectType::String,
        })
    );
}

#[test]
fn reference_and_content_contracts_validate_only_their_declared_families() {
    let entity_contract = worth_foundational::AspectContract::reference_entity(
        key("entity.parent"),
        identity(30),
        revision(1),
    );
    let content_contract = worth_foundational::AspectContract::content_ref(
        key("blob.preview"),
        identity(31),
        revision(1),
    );

    let entity_outcome = validate_aspect_value(
        &entity_contract,
        worth_foundational::AspectValue::EntityRef(worth_foundational::EntityId::new(
            worth_foundational::PartitionId::main(),
            1,
            0,
        ))
        .into(),
    );
    let content_outcome = validate_aspect_value(
        &content_contract,
        worth_foundational::AspectValue::ContentRef(ContentRefId(9)).into(),
    );
    let denied = validate_aspect_value(
        &entity_contract,
        worth_foundational::AspectValue::ContentRef(ContentRefId(9)).into(),
    );

    assert!(matches!(
        entity_outcome,
        worth_proof::TransitionOutcome::Success(_)
    ));
    assert!(matches!(
        content_outcome,
        worth_proof::TransitionOutcome::Success(_)
    ));
    assert_eq!(
        denied,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::ContractValidationDenial::ScalarTypeMismatch {
                expected: worth_foundational::ScalarAspectType::EntityRef,
                found: worth_foundational::ScalarAspectType::ContentRef,
            }
        )
    );
}

#[test]
fn opaque_contracts_fail_closed_for_scalar_or_struct_values() {
    let opaque_contract = worth_foundational::AspectContract::opaque_token(
        key("opaque.token"),
        identity(40),
        revision(1),
    );

    let scalar_denied = validate_aspect_value(
        &opaque_contract,
        worth_foundational::AspectValue::Int64(1).into(),
    );
    let struct_denied = validate_aspect_value(
        &opaque_contract,
        worth_foundational::StructAspectValue::new([])
            .expect("unique fields")
            .into(),
    );

    assert_eq!(
        scalar_denied,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::ContractValidationDenial::ScalarValueRequired
        )
    );
    assert_eq!(
        struct_denied,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::ContractValidationDenial::ScalarValueRequired
        )
    );
}

#[test]
fn scalar_contracts_reject_struct_inputs_before_shape_interpretation() {
    let contract = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(1),
        revision(1),
        worth_foundational::ScalarAspectType::Int64,
    );

    let outcome = validate_aspect_value(
        &contract,
        worth_foundational::StructAspectValue::new([])
            .expect("unique fields")
            .into(),
    );

    assert_eq!(
        outcome,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::ContractValidationDenial::ScalarValueRequired
        )
    );
}
