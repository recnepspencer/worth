use worth_foundational::validate_aspect_value;

use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision,
};

#[test]
fn struct_contract_validation_is_canonical_and_hostile_to_unknown_fields() {
    let title = field("title");
    let done = field("done");
    let shape = worth_foundational::StructAspectShape::new([
        worth_foundational::FieldDeclaration::new(
            done.clone(),
            worth_foundational::ScalarAspectType::Bool,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        worth_foundational::FieldDeclaration::new(
            title.clone(),
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
    ])
    .expect("unique fields");
    let contract = worth_foundational::AspectContract::struct_aspect(
        key("task.summary"),
        identity(2),
        revision(1),
        shape,
    );

    let value = worth_foundational::StructAspectValue::new([
        (
            title.clone(),
            worth_foundational::AspectValue::String(worth_foundational::CanonicalString::from(
                "Ship it",
            )),
        ),
        (done.clone(), worth_foundational::AspectValue::Bool(false)),
    ])
    .expect("unique fields");
    let outcome = validate_aspect_value(&contract, value.into());

    assert!(matches!(
        outcome,
        worth_proof::TransitionOutcome::Success(_)
    ));

    let unknown = field("surprise");
    let denied = validate_aspect_value(
        &contract,
        worth_foundational::StructAspectValue::new([
            (
                title,
                worth_foundational::AspectValue::String(worth_foundational::CanonicalString::from(
                    "Ship it",
                )),
            ),
            (done, worth_foundational::AspectValue::Bool(false)),
            (unknown.clone(), worth_foundational::AspectValue::Bool(true)),
        ])
        .expect("unique fields")
        .into(),
    );

    assert_eq!(
        denied,
        worth_proof::TransitionOutcome::Denied(
            worth_foundational::ContractValidationDenial::UnknownField(unknown)
        )
    );
}

#[test]
fn struct_field_order_is_canonical_across_construction_paths() {
    let a = worth_foundational::FieldKey::new("a").expect("valid field");
    let b = worth_foundational::FieldKey::new("b").expect("valid field");
    let left = worth_foundational::StructAspectValue::new([
        (b.clone(), worth_foundational::AspectValue::Int32(2)),
        (a.clone(), worth_foundational::AspectValue::Int32(1)),
    ])
    .expect("unique fields");
    let right = worth_foundational::StructAspectValue::new([
        (a.clone(), worth_foundational::AspectValue::Int32(1)),
        (b.clone(), worth_foundational::AspectValue::Int32(2)),
    ])
    .expect("unique fields");

    let left_fields: Vec<_> = left.fields().map(|(key, _)| key.as_str()).collect();

    assert_eq!(left, right);
    assert_eq!(left_fields, vec!["a", "b"]);
}

#[test]
fn struct_value_construction_rejects_duplicate_fields() {
    let title = field("title");

    let denied = worth_foundational::StructAspectValue::new([
        (
            title.clone(),
            worth_foundational::AspectValue::String(worth_foundational::CanonicalString::from(
                "first",
            )),
        ),
        (
            title.clone(),
            worth_foundational::AspectValue::String(worth_foundational::CanonicalString::from(
                "second",
            )),
        ),
    ]);

    assert_eq!(
        denied,
        Err(worth_foundational::StructAspectValueConstructionDenial::DuplicateField(title))
    );
}

#[test]
fn field_declaration_rejects_requirement_absence_drift() {
    let denied = worth_foundational::FieldDeclaration::new(
        field("title"),
        worth_foundational::ScalarAspectType::String,
        worth_foundational::FieldRequirement::Required,
        worth_foundational::AbsenceLaw::Optional,
        worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
    );

    assert_eq!(denied, None);
}
