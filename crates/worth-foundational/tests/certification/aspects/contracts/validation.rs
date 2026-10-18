use worth_foundational::{
    validate_aspect_value, AbsenceLaw, AspectContract, AspectValue, CanonicalF64, ContentRefId,
    ContractValidatedAspectValueView, ContractValidationDenial, EntityId, FieldDeclaration,
    FieldRequirement, PartitionId, ScalarAspectType, StructAspectShape, StructAspectValue,
};
use worth_proof::TransitionOutcome;

use crate::certification::foundational_vocabulary::{identity, key, revision};

#[test]
fn scalar_contract_validation_returns_proof_bearing_artifact() {
    let contract = AspectContract::scalar(
        key("temperature.celsius"),
        identity(1),
        revision(3),
        ScalarAspectType::Float64,
    );

    let outcome = validate_aspect_value(
        &contract,
        AspectValue::Float64(CanonicalF64::from_f64(21.0)).into(),
    );

    let TransitionOutcome::Success(artifact) = outcome else {
        panic!("expected validated scalar artifact");
    };

    match artifact.payload().view() {
        ContractValidatedAspectValueView::Scalar(value) => {
            assert_eq!(artifact.payload().key().as_str(), "temperature.celsius");
            assert_eq!(value.value_family(), ScalarAspectType::Float64);
            assert_eq!(artifact.payload().contract_revision(), revision(3));
        }
        ContractValidatedAspectValueView::Struct(_) => panic!("scalar contract produced struct"),
    }
}

#[test]
fn scalar_contract_validation_denies_wrong_width() {
    let contract = AspectContract::scalar(
        key("count"),
        identity(1),
        revision(1),
        ScalarAspectType::Int64,
    );

    let outcome = validate_aspect_value(&contract, AspectValue::Int32(9).into());

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(ContractValidationDenial::ScalarTypeMismatch {
            expected: ScalarAspectType::Int64,
            found: ScalarAspectType::Int32,
        })
    );
}

#[test]
fn struct_contract_validation_denies_missing_required_field() {
    let contract = task_summary_contract();

    let outcome = validate_aspect_value(
        &contract,
        StructAspectValue::new([(
            crate::certification::foundational_vocabulary::field("title"),
            AspectValue::String("Ship it".into()),
        )])
        .expect("unique fields")
        .into(),
    );

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(ContractValidationDenial::MissingRequiredField(
            crate::certification::foundational_vocabulary::field("done")
        ))
    );
}

#[test]
fn struct_contract_validation_denies_field_type_mismatch() {
    let contract = task_summary_contract();

    let outcome = validate_aspect_value(
        &contract,
        StructAspectValue::new([
            (
                crate::certification::foundational_vocabulary::field("title"),
                AspectValue::String("Ship it".into()),
            ),
            (
                crate::certification::foundational_vocabulary::field("done"),
                AspectValue::String("no".into()),
            ),
        ])
        .expect("unique fields")
        .into(),
    );

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(ContractValidationDenial::FieldTypeMismatch {
            field: crate::certification::foundational_vocabulary::field("done"),
            expected: ScalarAspectType::Bool,
            found: ScalarAspectType::String,
        })
    );
}

#[test]
fn reference_and_content_contracts_validate_only_their_declared_families() {
    let entity_contract =
        AspectContract::reference_entity(key("entity.parent"), identity(30), revision(1));
    let content_contract =
        AspectContract::content_ref(key("blob.preview"), identity(31), revision(1));

    let entity_outcome = validate_aspect_value(
        &entity_contract,
        AspectValue::EntityRef(EntityId::new(PartitionId::main(), 1, 0)).into(),
    );
    let content_outcome = validate_aspect_value(
        &content_contract,
        AspectValue::ContentRef(ContentRefId(9)).into(),
    );
    let denied = validate_aspect_value(
        &entity_contract,
        AspectValue::ContentRef(ContentRefId(9)).into(),
    );

    assert!(matches!(entity_outcome, TransitionOutcome::Success(_)));
    assert!(matches!(content_outcome, TransitionOutcome::Success(_)));
    assert_eq!(
        denied,
        TransitionOutcome::Denied(ContractValidationDenial::ScalarTypeMismatch {
            expected: ScalarAspectType::EntityRef,
            found: ScalarAspectType::ContentRef,
        })
    );
}

#[test]
fn opaque_contracts_fail_closed_for_scalar_or_struct_values() {
    let opaque_contract =
        AspectContract::opaque_token(key("opaque.token"), identity(40), revision(1));

    let scalar_denied = validate_aspect_value(&opaque_contract, AspectValue::Int64(1).into());
    let struct_denied = validate_aspect_value(
        &opaque_contract,
        StructAspectValue::new([]).expect("unique fields").into(),
    );

    assert_eq!(
        scalar_denied,
        TransitionOutcome::Denied(ContractValidationDenial::ScalarValueRequired)
    );
    assert_eq!(
        struct_denied,
        TransitionOutcome::Denied(ContractValidationDenial::ScalarValueRequired)
    );
}

#[test]
fn scalar_contracts_reject_struct_inputs_before_shape_interpretation() {
    let contract = AspectContract::scalar(
        key("count"),
        identity(1),
        revision(1),
        ScalarAspectType::Int64,
    );

    let outcome = validate_aspect_value(
        &contract,
        StructAspectValue::new([]).expect("unique fields").into(),
    );

    assert_eq!(
        outcome,
        TransitionOutcome::Denied(ContractValidationDenial::ScalarValueRequired)
    );
}

fn task_summary_contract() -> AspectContract {
    let shape = StructAspectShape::new([
        FieldDeclaration::new(
            crate::certification::foundational_vocabulary::field("title"),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        FieldDeclaration::new(
            crate::certification::foundational_vocabulary::field("done"),
            ScalarAspectType::Bool,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
    ])
    .expect("unique fields");

    AspectContract::struct_aspect(key("task.summary"), identity(2), revision(1), shape)
}
