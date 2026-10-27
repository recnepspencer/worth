use worth_foundational::facade::{
    aspects, AspectBinding, AspectIdentity, AspectKey, ScalarAspectType,
};
use worth_relational::facade::config::{CascadeDeletePolicy, CrossContextPolicy};
use worth_relational::facade::identity::KindId;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationIntegrityDeclarations, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};

pub(super) fn fixture_runtime() -> RelationalRuntime {
    let state_aspects = KindAspectContractDeclarations::new(vec![
        string_field_aspect("AlphaState", "Alpha", 101),
        string_field_aspect("BetaState", "Beta", 102),
    ]);
    let registry = [
        (KindId::new(11), "observed.update", state_aspects),
        (
            KindId::new(12),
            "observed.delete",
            KindAspectContractDeclarations::default(),
        ),
        (
            KindId::new(13),
            "observed.from",
            KindAspectContractDeclarations::default(),
        ),
        (
            KindId::new(14),
            "observed.to",
            KindAspectContractDeclarations::default(),
        ),
    ]
    .into_iter()
    .try_fold(
        RelationalSchemaRegistry::new(),
        |registry, (kind_id, name, aspects)| {
            registry.register_entity_kind(EntityKindRegistration {
                kind_id,
                kind_name: name.to_owned(),
                schema_id: SchemaId("provider-effect-proof".to_owned()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: aspects,
            })
        },
    )
    .and_then(|registry| {
        registry.register_relation_kind(RelationKindRegistration {
            kind_id: KindId::new(31),
            kind_name: "observed.edge".to_owned(),
            schema_id: SchemaId("provider-effect-proof".to_owned()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
            relation_integrity: RelationIntegrityDeclarations::default(),
        })
    })
    .expect("fixture schema is valid");
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration")
}

fn string_field_aspect(aspect: &str, field: &str, identity: u64) -> DeclaredAspectContractBinding {
    let aspect = AspectKey::new(aspect).expect("valid fixture aspect");
    let shape = aspects()
        .struct_fields()
        .required(field, ScalarAspectType::String)
        .finish()
        .expect("valid fixture struct aspect");
    DeclaredAspectContractBinding {
        binding: AspectBinding::EntityField {
            field: worth_foundational::facade::FieldKey::new(field).expect("valid fixture field"),
        },
        contract: aspects()
            .contract()
            .for_key(aspect)
            .identified_by(AspectIdentity(identity))
            .at_revision(aspects().vocabulary().revision(1))
            .struct_aspect(shape),
    }
}
