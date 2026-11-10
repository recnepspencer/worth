//! The original declared schema/policy, including its optional name identity.
use definition::{aspect_key, field_key};
use std::sync::Arc;
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy},
    identity::KindId,
    merge::*,
    runtime::{RelationalRuntime, RelationalRuntimeApi, RelationalRuntimeBuilder},
    schema::*,
};
use worth_testkit_relational::{aspect_storage::definition, durable_medium::TestDurableMedium};
fn materialize((binding, contract): definition::DeclaredAspect) -> DeclaredAspectContractBinding {
    DeclaredAspectContractBinding { binding, contract }
}
fn entity_field_aspect(
    aspect: worth_foundational::facade::AspectKey,
    field: worth_foundational::facade::FieldKey,
) -> DeclaredAspectContractBinding {
    materialize(definition::entity_field_aspect(aspect, field))
}
fn relation_field_aspect(
    aspect: worth_foundational::facade::AspectKey,
    field: worth_foundational::facade::FieldKey,
) -> DeclaredAspectContractBinding {
    materialize(definition::relation_field_aspect(aspect, field))
}
fn relation_source_aspect() -> DeclaredAspectContractBinding {
    materialize(definition::relation_source_aspect())
}
fn relation_target_aspect() -> DeclaredAspectContractBinding {
    materialize(definition::relation_target_aspect())
}
pub(super) fn policy_builder(
    policy: AspectMergePolicyKind,
    identify_by_name: bool,
) -> RelationalRuntimeBuilder {
    let name_key = aspect_key("name");
    let status_key = aspect_key("status");
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(name_key.clone(), field_key("name")),
                entity_field_aspect(status_key.clone(), field_key("status")),
            ])
            .with_identity_declarations(if identify_by_name {
                vec![IdentityBasisDeclaration {
                    scope: IdentityBasisScope::AspectKey(name_key.clone()),
                    basis: IdentityBasisKind::DeclaredKeySet(
                        Arc::<[worth_foundational::facade::AspectKey]>::from([name_key]).into(),
                    ),
                }]
            } else {
                Vec::new()
            })
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: status_key,
                policy,
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
        .expect("schema-declared merge policy registry");
    RelationalRuntimeApi::builder().schema_registry(registry)
}
pub(super) fn runtime_with_relation_identity_registry(
    medium: &Arc<TestDurableMedium>,
) -> RelationalRuntime {
    let label_key = aspect_key("label");
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .and_then(|registry| {
            registry.register_relation_kind(RelationKindRegistration {
                kind_id: KindId(2),
                kind_name: "test.relation".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    relation_field_aspect(label_key.clone(), field_key("label")),
                    relation_source_aspect(),
                    relation_target_aspect(),
                ])
                .with_identity_declarations(vec![IdentityBasisDeclaration {
                    scope: IdentityBasisScope::AspectKey(label_key.clone()),
                    basis: IdentityBasisKind::DeclaredKeySet(
                        Arc::<[worth_foundational::facade::AspectKey]>::from([label_key]).into(),
                    ),
                }]),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("topology identity registry");

    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build_durable(medium.attachment())
        .expect("valid native runtime configuration")
}
