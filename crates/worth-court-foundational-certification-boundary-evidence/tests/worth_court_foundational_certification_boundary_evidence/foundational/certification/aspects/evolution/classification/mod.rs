use worth_foundational::classify_aspect_contract_evolution;

use {
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key,
    worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision,
};

#[test]
fn absence_null_default_and_clear_are_distinct_surface_states() {
    assert_ne!(
        worth_foundational::AbsenceLaw::Required,
        worth_foundational::AbsenceLaw::Optional
    );
    assert_ne!(
        worth_foundational::AbsenceLaw::Optional,
        worth_foundational::AbsenceLaw::Defaulted
    );
    assert_ne!(
        worth_foundational::AspectValue::Null.value_family(),
        worth_foundational::ScalarAspectType::String
    );
}

#[test]
fn evolution_classification_is_deterministic() {
    let base = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(9),
        revision(1),
        worth_foundational::ScalarAspectType::Int32,
    );
    let widened = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(9),
        revision(2),
        worth_foundational::ScalarAspectType::Int64,
    );
    let narrowed = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(9),
        revision(3),
        worth_foundational::ScalarAspectType::Int8,
    );
    let incompatible = worth_foundational::AspectContract::scalar(
        key("other"),
        identity(10),
        revision(1),
        worth_foundational::ScalarAspectType::Int32,
    );

    assert_eq!(
        base.classify_evolution_to(&widened).kind(),
        worth_foundational::AspectEvolutionKind::Widening
    );
    assert_eq!(
        base.classify_evolution_to(&narrowed).kind(),
        worth_foundational::AspectEvolutionKind::Narrowing
    );
    assert_eq!(
        base.classify_evolution_to(&incompatible).kind(),
        worth_foundational::AspectEvolutionKind::Incompatible
    );
}

#[test]
fn evolution_classification_can_be_carried_as_a_proof_artifact() {
    let previous = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(9),
        revision(1),
        worth_foundational::ScalarAspectType::Int32,
    );
    let next = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(9),
        revision(2),
        worth_foundational::ScalarAspectType::Int64,
    );

    let classified = classify_aspect_contract_evolution(previous, next);
    let payload = classified.payload();

    assert_eq!(payload.previous().key(), &key("count"));
    assert_eq!(payload.next().revision(), revision(2));
    assert_eq!(
        payload.verdict().kind(),
        worth_foundational::AspectEvolutionKind::Widening
    );
}

#[test]
fn equivalence_basis_is_declared_before_comparison_claims() {
    let scalar = worth_foundational::AspectContract::scalar(
        key("count"),
        identity(1),
        revision(1),
        worth_foundational::ScalarAspectType::Int64,
    );
    let title_field = worth_foundational::FieldDeclaration::new(
        field("title"),
        worth_foundational::ScalarAspectType::String,
        worth_foundational::FieldRequirement::Required,
        worth_foundational::AbsenceLaw::Required,
        worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("coherent field law");
    let shape = worth_foundational::StructAspectShape::new([title_field]).expect("unique fields");
    let structured = worth_foundational::AspectContract::struct_aspect(
        key("task"),
        identity(2),
        revision(1),
        shape,
    );

    assert_eq!(
        scalar.equivalence(),
        worth_foundational::AspectEquivalenceBasis::ExactCanonicalValue
    );
    assert_eq!(
        structured.equivalence(),
        worth_foundational::AspectEquivalenceBasis::DeclaredStructFields
    );
}

#[test]
fn required_struct_field_addition_is_not_additive() {
    let base_shape =
        worth_foundational::StructAspectShape::new([worth_foundational::FieldDeclaration::new(
            field("title"),
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law")])
        .expect("unique fields");
    let expanded_shape = worth_foundational::StructAspectShape::new([
        worth_foundational::FieldDeclaration::new(
            field("title"),
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        worth_foundational::FieldDeclaration::new(
            field("owner"),
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
    ])
    .expect("unique fields");
    let base = worth_foundational::AspectContract::struct_aspect(
        key("task"),
        identity(8),
        revision(1),
        base_shape,
    );
    let expanded = worth_foundational::AspectContract::struct_aspect(
        key("task"),
        identity(8),
        revision(2),
        expanded_shape,
    );

    assert_eq!(
        base.classify_evolution_to(&expanded).kind(),
        worth_foundational::AspectEvolutionKind::Incompatible
    );
}
