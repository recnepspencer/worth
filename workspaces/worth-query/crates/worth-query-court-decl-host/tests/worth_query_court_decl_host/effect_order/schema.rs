use super::*;
pub const KIND: KindId = KindId(41);
pub fn basis(name: &str) -> PortableAspectContractBasis {
    PortableAspectContractBasis::new(
        AspectKey::new(name).unwrap(),
        AspectIdentity(if name == "Alias" { 401 } else { 402 }),
        AspectContractRevision(1),
    )
}
pub fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new(aspect).unwrap(),
        CanonicalFieldPath::single(FieldKey::new(field).unwrap()),
    )
}
pub fn runtime() -> RelationalRuntime {
    let bindings = [
        ("Alias", "Unique", ScalarAspectType::String),
        ("Marker", "Count", ScalarAspectType::UInt64),
    ]
    .into_iter()
    .map(|(name, field, scalar)| {
        let basis = basis(name);
        DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: FieldKey::new(field).unwrap(),
            },
            contract: aspects()
                .contract()
                .for_key(basis.key().clone())
                .identified_by(basis.identity())
                .at_revision(basis.revision())
                .struct_with(
                    aspects()
                        .struct_fields()
                        .required(field, scalar)
                        .finish()
                        .unwrap(),
                    AspectMaskContract::struct_fields(),
                    AbsenceLaw::Optional,
                    AspectEquivalenceBasis::DeclaredStructFields,
                    AspectEvolutionPolicy::AdditiveFieldsAllowed,
                ),
        }
    })
    .collect();
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KIND,
            kind_name: "native.effect-order".into(),
            schema_id: SchemaId("effect-order.v1".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(bindings),
        })
        .unwrap();
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .invariant_catalog(InvariantCatalog {
            registrations: vec![InvariantRegistration::mutation_sensitive_blocking(
                InvariantRule::unique_entity_aspect_field(
                    AspectKey::new("Alias").unwrap(),
                    FieldKey::new("Unique").unwrap(),
                ),
            )],
        })
        .build()
        .unwrap()
}
