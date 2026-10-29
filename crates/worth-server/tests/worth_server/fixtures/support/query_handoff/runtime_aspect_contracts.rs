use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType, StructAspectShape,
};

pub(crate) fn query_handoff_aspect_contracts() -> Vec<AspectContract> {
    vec![
        required_string_contract("identity", 0x5753_0001, "id"),
        required_string_contract("title", 0x5753_0002, "value"),
        required_string_contract("profile", 0x5753_0003, "display_name"),
    ]
}

fn required_string_contract(aspect: &str, identity: u64, field: &str) -> AspectContract {
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
