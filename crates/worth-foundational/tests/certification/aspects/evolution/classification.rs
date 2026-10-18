use worth_foundational::{
    classify_aspect_contract_evolution, AbsenceLaw, AspectContract, AspectEquivalenceBasis,
    AspectEvolutionKind, AspectValue, FieldDeclaration, FieldRequirement, ScalarAspectType,
    StructAspectShape,
};

use crate::certification::foundational_vocabulary::{field, identity, key, revision};

#[test]
fn absence_null_default_and_clear_are_distinct_surface_states() {
    assert_ne!(AbsenceLaw::Required, AbsenceLaw::Optional);
    assert_ne!(AbsenceLaw::Optional, AbsenceLaw::Defaulted);
    assert_ne!(AspectValue::Null.value_family(), ScalarAspectType::String);
}

#[test]
fn evolution_classification_is_deterministic() {
    let base = AspectContract::scalar(
        key("count"),
        identity(9),
        revision(1),
        ScalarAspectType::Int32,
    );
    let widened = AspectContract::scalar(
        key("count"),
        identity(9),
        revision(2),
        ScalarAspectType::Int64,
    );
    let narrowed = AspectContract::scalar(
        key("count"),
        identity(9),
        revision(3),
        ScalarAspectType::Int8,
    );
    let incompatible = AspectContract::scalar(
        key("other"),
        identity(10),
        revision(1),
        ScalarAspectType::Int32,
    );

    assert_eq!(
        base.classify_evolution_to(&widened).kind(),
        AspectEvolutionKind::Widening
    );
    assert_eq!(
        base.classify_evolution_to(&narrowed).kind(),
        AspectEvolutionKind::Narrowing
    );
    assert_eq!(
        base.classify_evolution_to(&incompatible).kind(),
        AspectEvolutionKind::Incompatible
    );
}

#[test]
fn evolution_classification_can_be_carried_as_a_proof_artifact() {
    let previous = AspectContract::scalar(
        key("count"),
        identity(9),
        revision(1),
        ScalarAspectType::Int32,
    );
    let next = AspectContract::scalar(
        key("count"),
        identity(9),
        revision(2),
        ScalarAspectType::Int64,
    );

    let classified = classify_aspect_contract_evolution(previous, next);
    let payload = classified.payload();

    assert_eq!(payload.previous().key(), &key("count"));
    assert_eq!(payload.next().revision(), revision(2));
    assert_eq!(payload.verdict().kind(), AspectEvolutionKind::Widening);
}

#[test]
fn equivalence_basis_is_declared_before_comparison_claims() {
    let scalar = AspectContract::scalar(
        key("count"),
        identity(1),
        revision(1),
        ScalarAspectType::Int64,
    );
    let title_field = FieldDeclaration::new(
        field("title"),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("coherent field law");
    let shape = StructAspectShape::new([title_field]).expect("unique fields");
    let structured = AspectContract::struct_aspect(key("task"), identity(2), revision(1), shape);

    assert_eq!(
        scalar.equivalence(),
        AspectEquivalenceBasis::ExactCanonicalValue
    );
    assert_eq!(
        structured.equivalence(),
        AspectEquivalenceBasis::DeclaredStructFields
    );
}

#[test]
fn required_struct_field_addition_is_not_additive() {
    let base_shape = StructAspectShape::new([FieldDeclaration::new(
        field("title"),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("coherent field law")])
    .expect("unique fields");
    let expanded_shape = StructAspectShape::new([
        FieldDeclaration::new(
            field("title"),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        FieldDeclaration::new(
            field("owner"),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
    ])
    .expect("unique fields");
    let base = AspectContract::struct_aspect(key("task"), identity(8), revision(1), base_shape);
    let expanded =
        AspectContract::struct_aspect(key("task"), identity(8), revision(2), expanded_shape);

    assert_eq!(
        base.classify_evolution_to(&expanded).kind(),
        AspectEvolutionKind::Incompatible
    );
}
