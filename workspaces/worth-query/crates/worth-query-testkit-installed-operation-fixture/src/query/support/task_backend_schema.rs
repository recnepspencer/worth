pub use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType, StructAspectShape,
};
pub use worth_query::facade::consumer_kit::WorthQueryTestBackendSchema;

pub fn task_backend_schema() -> WorthQueryTestBackendSchema {
    WorthQueryTestBackendSchema::single_collection("Task")
        .aspect_contract(required_string_contract("identity", 0x5751_4001, "id"))
        .expect("identity contract should install")
        .aspect_contract(required_string_contract("title", 0x5751_4002, "value"))
        .expect("title contract should install")
        .aspect("identity.id", "identity.id")
        .expect("identity mapping should admit")
        .aspect("title.value", "title.value")
        .expect("title mapping should admit")
}

pub fn required_string_contract(aspect: &str, identity: u64, field: &str) -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new(field).expect("field key should admit"),
        ScalarAspectType::String,
        FieldRequirement::Required,
        AbsenceLaw::Required,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("field declaration should admit");
    AspectContract::struct_aspect(
        AspectKey::new(aspect).expect("aspect key should admit"),
        AspectIdentity(identity),
        AspectContractRevision(1),
        StructAspectShape::new([field]).expect("struct aspect shape should admit"),
    )
}
