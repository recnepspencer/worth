pub use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, FieldDeclaration, FieldKey, FieldRequirement, ScalarAspectType, StructAspectShape,
};

pub fn public_bridge_aspect_contracts() -> Vec<AspectContract> {
    vec![
        string_contract(
            "identity",
            0x5751_4201,
            "id",
            FieldRequirement::Required,
            AbsenceLaw::Required,
        ),
        string_contract(
            "title",
            0x5751_4202,
            "value",
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
        ),
        edge_contract(),
        string_contract(
            "kind",
            0x5751_4204,
            "value",
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
        ),
        string_contract(
            "source",
            0x5751_4205,
            "id",
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
        ),
        string_contract(
            "target",
            0x5751_4206,
            "id",
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
        ),
        string_contract(
            "status",
            0x5751_4207,
            "value",
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
        ),
        profile_contract(),
        string_contract(
            "description",
            0x5751_4209,
            "value",
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
        ),
    ]
}

pub fn edge_contract() -> AspectContract {
    let fields = [
        ("kind", ScalarAspectType::String),
        ("source_identity", ScalarAspectType::EntityRef),
        ("target_identity", ScalarAspectType::EntityRef),
    ]
    .map(|(field, field_type)| {
        FieldDeclaration::new(
            FieldKey::new(field).expect("public bridge field must admit"),
            field_type,
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("public bridge field law must be coherent")
    });
    AspectContract::struct_aspect(
        AspectKey::new("edge").expect("public bridge aspect must admit"),
        AspectIdentity(0x5751_4203),
        AspectContractRevision(1),
        StructAspectShape::new(fields).expect("public bridge fields must be unique"),
    )
}

pub fn profile_contract() -> AspectContract {
    let fields = [
        ("display_name", ScalarAspectType::String),
        ("age", ScalarAspectType::Int64),
    ]
    .map(|(field, field_type)| {
        FieldDeclaration::new(
            FieldKey::new(field).expect("public bridge field must admit"),
            field_type,
            FieldRequirement::Optional,
            AbsenceLaw::Optional,
            AspectEvolutionPolicy::ExplicitBreakRequired,
        )
        .expect("public bridge field law must be coherent")
    });
    AspectContract::struct_aspect(
        AspectKey::new("profile").expect("public bridge aspect must admit"),
        AspectIdentity(0x5751_4208),
        AspectContractRevision(1),
        StructAspectShape::new(fields).expect("public bridge fields must be unique"),
    )
}

pub fn string_contract(
    aspect: &str,
    identity: u64,
    field: &str,
    requirement: FieldRequirement,
    absence: AbsenceLaw,
) -> AspectContract {
    let field = FieldDeclaration::new(
        FieldKey::new(field).expect("public bridge field must admit"),
        ScalarAspectType::String,
        requirement,
        absence,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .expect("public bridge field law must be coherent");

    AspectContract::struct_aspect(
        AspectKey::new(aspect).expect("public bridge aspect must admit"),
        AspectIdentity(identity),
        AspectContractRevision(1),
        StructAspectShape::new([field]).expect("public bridge field must be unique"),
    )
}
