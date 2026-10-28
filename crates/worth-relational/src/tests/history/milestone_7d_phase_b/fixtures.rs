use crate::facade::runtime::RelationalRuntimeApi;
use crate::merge::data::AspectMergePolicyDeclaration;
use crate::schema::data::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration, SchemaId,
    SchemaVersionId,
};
use crate::tests::support::{entity_field_aspect, field_key};
use crate::{
    config::data::{CascadeDeletePolicy, CrossContextPolicy},
    facade::identity::KindId,
    facade::merge::{
        AspectMergePolicyKind, IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
    },
    schema::data::RelationalSchemaRegistry,
};
use worth_foundational::facade::AspectKey;

pub(super) fn runtime_with_name_merge_policy(
    merge_policy: AspectMergePolicyKind,
) -> crate::facade::runtime::RelationalRuntime {
    let name_key = AspectKey::new("name").unwrap();
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(name_key.clone(), field_key("name")),
            ])
            .with_identity_declarations(vec![IdentityBasisDeclaration {
                scope: IdentityBasisScope::AspectKey(name_key.clone()),
                basis: IdentityBasisKind::DeclaredKeySet(vec![name_key.clone()].into()),
            }])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: name_key,
                policy: merge_policy,
            }]),
        })
        .and_then(|registry: RelationalSchemaRegistry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::default(),
            })
        })
        .expect("schema registry");
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration")
}
