//! Declared aspect merge-policy registration, shared by its denial units.
mod aspect_merge_policy_denials;

use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::facade::identity::KindId;
use crate::facade::merge::{
    AspectMergePolicyKind, IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope,
};
use crate::merge::data::AspectMergePolicyDeclaration;
use crate::schema::data::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaRegistryError, SchemaRegistryErrorClass,
    SchemaVersionId,
};
use crate::tests::support::{entity_field_aspect, entity_summary_struct_aspect, field_key};
use worth_foundational::facade::AspectKey;

fn register_aspect_field_merge_policy(
    aspect_key: AspectKey,
    value_aspect: crate::schema::data::DeclaredAspectContractBinding,
    merge_policy: AspectMergePolicyKind,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let name_key = AspectKey::new("name").unwrap();
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(name_key.clone(), field_key("name")),
                value_aspect,
            ])
            .with_identity_declarations(vec![IdentityBasisDeclaration {
                scope: IdentityBasisScope::AspectKey(name_key.clone()),
                basis: IdentityBasisKind::DeclaredKeySet(vec![name_key].into()),
            }])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key,
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
}
