use std::sync::Arc;

use crate::facade::identity::KindId;
use crate::facade::merge::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
    IdentityBasisKind, IdentityBasisScope,
};
use crate::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use crate::schema::data::RelationIntegrityDeclarations;
use crate::tests::support::{
    entity_field_aspect, CascadeDeletePolicy, CrossContextPolicy, RelationalRuntimeProfile,
};
use worth_foundational::facade::AspectKey;

pub(super) fn prefer_richer_registry() -> RelationalSchemaRegistry {
    let name_key = AspectKey::new("name").unwrap();
    let status_key = AspectKey::new("status").unwrap();
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                ),
                entity_field_aspect(
                    status_key.clone(),
                    crate::tests::support::field_key("status"),
                ),
            ])
            .with_identity_declarations(vec![IdentityBasisDeclaration {
                scope: IdentityBasisScope::AspectKey(name_key.clone()),
                basis: IdentityBasisKind::DeclaredKeySet(Arc::from([name_key.clone()])),
            }])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: status_key,
                policy: AspectMergePolicyKind::PreferRicher,
            }]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("prefer-richer registry")
}

pub(super) fn drifted_schema_registry() -> RelationalSchemaRegistry {
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(2),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(
                    crate::tests::support::aspect_key("name"),
                    crate::tests::support::field_key("name"),
                ),
                entity_field_aspect(
                    crate::tests::support::aspect_key("status"),
                    crate::tests::support::field_key("status"),
                ),
            ]),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(2),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("drifted schema registry")
}

pub(super) fn durable_runtime_with_registry(
    registry: RelationalSchemaRegistry,
    medium: std::sync::Arc<crate::tests::support::TestDurableMedium>,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .build_durable(medium.attachment())
}
