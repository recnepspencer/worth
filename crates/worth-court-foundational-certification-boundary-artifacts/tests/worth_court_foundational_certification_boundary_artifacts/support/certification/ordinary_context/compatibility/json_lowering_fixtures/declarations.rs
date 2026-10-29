pub(crate) use {worth_foundational::AbsenceLaw, worth_foundational::AspectContract, worth_foundational::AspectFieldLocator, worth_foundational::AspectKey, worth_foundational::AspectLocator, worth_foundational::BoundarySourceLocator, worth_foundational::CanonicalFieldPath, worth_foundational::FieldDeclaration, worth_foundational::FieldKey, worth_foundational::FieldRequirement, worth_foundational::JsonCompatibilityAspectInput, worth_foundational::LocatorAuthority, worth_foundational::ScalarAspectType, worth_foundational::StructAspectShape};

pub(crate) use {worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::field, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::identity, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::key, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::revision, worth_testkit_foundational::certification::ordinary_context::foundational_vocabulary::scalar_contract};

pub(crate) fn task_summary_contract() -> AspectContract {
    let shape = worth_foundational::StructAspectShape::new([
        worth_foundational::FieldDeclaration::new(
            field("title"),
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        worth_foundational::FieldDeclaration::new(
            field("done"),
            worth_foundational::ScalarAspectType::Bool,
            worth_foundational::FieldRequirement::Required,
            worth_foundational::AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        worth_foundational::FieldDeclaration::new(
            field("note"),
            worth_foundational::ScalarAspectType::String,
            worth_foundational::FieldRequirement::Optional,
            worth_foundational::AbsenceLaw::Optional,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
    ])
    .expect("unique fields");

    worth_foundational::AspectContract::struct_aspect(key("task.summary"), identity(20), revision(1), shape)
}

pub(crate) fn source_for(name: &str) -> BoundarySourceLocator {
    worth_foundational::BoundarySourceLocator::aspect(worth_foundational::AspectLocator::new(
        worth_foundational::LocatorAuthority::SupportOnly,
        worth_foundational::AspectKey::new(name).expect("valid aspect key"),
    ))
}

pub(crate) fn field_source_for(aspect: &str, field: &str) -> BoundarySourceLocator {
    worth_foundational::BoundarySourceLocator::aspect_field(worth_foundational::AspectFieldLocator::new(
        worth_foundational::LocatorAuthority::SupportOnly,
        worth_foundational::AspectKey::new(aspect).expect("valid aspect key"),
        worth_foundational::CanonicalFieldPath::single(worth_foundational::FieldKey::new(field).expect("valid field key")),
    ))
}

pub(crate) fn scalar_input(
    name: &str,
    scalar: ScalarAspectType,
    value: serde_json::Value,
) -> JsonCompatibilityAspectInput {
    worth_foundational::JsonCompatibilityAspectInput::new(scalar_contract(name, 1, scalar), source_for(name), value)
}