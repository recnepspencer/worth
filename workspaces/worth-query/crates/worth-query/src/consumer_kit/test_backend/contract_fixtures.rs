use super::WorthQueryTestBackendSchema;
use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType, StructAspectShape,
};

#[cfg(test)]
pub(crate) fn task_schema() -> WorthQueryTestBackendSchema {
    WorthQueryTestBackendSchema::single_collection("Task")
        .aspect_contract(required_string_struct_contract(
            "identity",
            0x5751_2001,
            "id",
        ))
        .unwrap()
        .aspect_contract(required_string_struct_contract(
            "title",
            0x5751_2002,
            "value",
        ))
        .unwrap()
        .aspect("identity.id", "identity.id")
        .expect("identity aspect")
        .aspect("title.value", "title.value")
        .expect("title aspect")
}

#[cfg(test)]
fn required_string_struct_contract(aspect: &str, identity: u64, field: &str) -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new(field).unwrap(),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap();
    AspectContract::struct_aspect(
        AspectKey::new(aspect).unwrap(),
        AspectIdentity(identity),
        AspectContractRevision(1),
        StructAspectShape::new([field]).unwrap(),
    )
}
