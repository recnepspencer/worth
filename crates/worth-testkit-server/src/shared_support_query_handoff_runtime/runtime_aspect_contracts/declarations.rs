pub use {worth_foundational::facade::AbsenceLaw, worth_foundational::facade::AspectContract, worth_foundational::facade::AspectContractRevision, worth_foundational::facade::AspectEvolutionPolicy, worth_foundational::facade::AspectIdentity, worth_foundational::facade::AspectKey, worth_foundational::facade::FieldDeclaration, worth_foundational::facade::FieldKey, worth_foundational::facade::FieldRequirement, worth_foundational::facade::ScalarAspectType, worth_foundational::facade::StructAspectShape};

pub fn query_handoff_aspect_contracts() -> Vec<AspectContract> {
    vec![
        required_string_contract("identity", 0x5753_0001, "id"),
        required_string_contract("title", 0x5753_0002, "value"),
        required_string_contract("profile", 0x5753_0003, "display_name"),
    ]
}

pub fn required_string_contract(aspect: &str, identity: u64, field: &str) -> AspectContract {
    let field = worth_foundational::facade::FieldDeclaration::new(
        worth_foundational::facade::FieldKey::new(field).expect("field key should admit"),
        worth_foundational::facade::ScalarAspectType::String,
        worth_foundational::facade::FieldRequirement::Required,
        worth_foundational::facade::AbsenceLaw::Required,
        worth_foundational::facade::AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("field declaration should admit");
    worth_foundational::facade::AspectContract::struct_aspect(
        worth_foundational::facade::AspectKey::new(aspect).expect("aspect key should admit"),
        AspectIdentity(identity),
        AspectContractRevision(1),
        worth_foundational::facade::StructAspectShape::new([field]).expect("struct aspect shape should admit"),
    )
}