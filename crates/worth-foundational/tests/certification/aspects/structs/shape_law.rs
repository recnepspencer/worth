use worth_foundational::{
    validate_aspect_value, AbsenceLaw, AspectContract, AspectValue, CanonicalString,
    ContractValidationDenial, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType,
    StructAspectShape, StructAspectValue, StructAspectValueConstructionDenial,
};
use worth_proof::TransitionOutcome;

use crate::certification::foundational_vocabulary::{field, identity, key, revision};

#[test]
fn struct_contract_validation_is_canonical_and_hostile_to_unknown_fields() {
    let title = field("title");
    let done = field("done");
    let shape = StructAspectShape::new([
        FieldDeclaration::new(
            done.clone(),
            ScalarAspectType::Bool,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        FieldDeclaration::new(
            title.clone(),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
    ])
    .expect("unique fields");
    let contract =
        AspectContract::struct_aspect(key("task.summary"), identity(2), revision(1), shape);

    let value = StructAspectValue::new([
        (
            title.clone(),
            AspectValue::String(CanonicalString::from("Ship it")),
        ),
        (done.clone(), AspectValue::Bool(false)),
    ])
    .expect("unique fields");
    let outcome = validate_aspect_value(&contract, value.into());

    assert!(matches!(outcome, TransitionOutcome::Success(_)));

    let unknown = field("surprise");
    let denied = validate_aspect_value(
        &contract,
        StructAspectValue::new([
            (title, AspectValue::String(CanonicalString::from("Ship it"))),
            (done, AspectValue::Bool(false)),
            (unknown.clone(), AspectValue::Bool(true)),
        ])
        .expect("unique fields")
        .into(),
    );

    assert_eq!(
        denied,
        TransitionOutcome::Denied(ContractValidationDenial::UnknownField(unknown))
    );
}

#[test]
fn struct_field_order_is_canonical_across_construction_paths() {
    let a = FieldKey::new("a").expect("valid field");
    let b = FieldKey::new("b").expect("valid field");
    let left = StructAspectValue::new([
        (b.clone(), AspectValue::Int32(2)),
        (a.clone(), AspectValue::Int32(1)),
    ])
    .expect("unique fields");
    let right = StructAspectValue::new([
        (a.clone(), AspectValue::Int32(1)),
        (b.clone(), AspectValue::Int32(2)),
    ])
    .expect("unique fields");

    let left_fields: Vec<_> = left.fields().map(|(key, _)| key.as_str()).collect();

    assert_eq!(left, right);
    assert_eq!(left_fields, vec!["a", "b"]);
}

#[test]
fn struct_value_construction_rejects_duplicate_fields() {
    let title = field("title");

    let denied = StructAspectValue::new([
        (
            title.clone(),
            AspectValue::String(CanonicalString::from("first")),
        ),
        (
            title.clone(),
            AspectValue::String(CanonicalString::from("second")),
        ),
    ]);

    assert_eq!(
        denied,
        Err(StructAspectValueConstructionDenial::DuplicateField(title))
    );
}

#[test]
fn field_declaration_rejects_requirement_absence_drift() {
    let denied = FieldDeclaration::new(
        field("title"),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Optional,
        worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
    );

    assert_eq!(denied, None);
}
