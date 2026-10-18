use worth_foundational::{
    AbsenceLaw, AspectContract, AspectFieldLocator, AspectKey, AspectLocator,
    BoundarySourceLocator, CanonicalFieldPath, FieldDeclaration, FieldKey, FieldRequirement,
    JsonCompatibilityAspectInput, LocatorAuthority, ScalarAspectType, StructAspectShape,
};

use crate::certification::foundational_vocabulary::{
    field, identity, key, revision, scalar_contract,
};

pub(super) fn task_summary_contract() -> AspectContract {
    let shape = StructAspectShape::new([
        FieldDeclaration::new(
            field("title"),
            ScalarAspectType::String,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        FieldDeclaration::new(
            field("done"),
            ScalarAspectType::Bool,
            FieldRequirement::Required,
            AbsenceLaw::Required,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
        FieldDeclaration::new(
            field("note"),
            ScalarAspectType::String,
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
            worth_foundational::AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("coherent field law"),
    ])
    .expect("unique fields");

    AspectContract::struct_aspect(key("task.summary"), identity(20), revision(1), shape)
}

pub(super) fn source_for(name: &str) -> BoundarySourceLocator {
    BoundarySourceLocator::aspect(AspectLocator::new(
        LocatorAuthority::SupportOnly,
        AspectKey::new(name).expect("valid aspect key"),
    ))
}

pub(super) fn field_source_for(aspect: &str, field: &str) -> BoundarySourceLocator {
    BoundarySourceLocator::aspect_field(AspectFieldLocator::new(
        LocatorAuthority::SupportOnly,
        AspectKey::new(aspect).expect("valid aspect key"),
        CanonicalFieldPath::single(FieldKey::new(field).expect("valid field key")),
    ))
}

pub(super) fn scalar_input(
    name: &str,
    scalar: ScalarAspectType,
    value: serde_json::Value,
) -> JsonCompatibilityAspectInput {
    JsonCompatibilityAspectInput::new(scalar_contract(name, 1, scalar), source_for(name), value)
}
