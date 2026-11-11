//! Declared merge-policy and topology-identity schemas of the execution courts.
pub(super) use super::merge_witness_schema::entity_field_aspect;
use super::merge_witness_schema::{materialize, relation_identity_registry};
use std::sync::Arc;
use worth_foundational::facade::{AspectKey, FieldKey};
use worth_relational::facade::{
    config::{CascadeDeletePolicy, CrossContextPolicy, RelationalRuntimeProfile},
    identity::KindId,
    merge::{
        AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
        IdentityBasisKind, IdentityBasisScope,
    },
    runtime::{RelationalRuntime, RelationalRuntimeApi},
    schema::{
        DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
        RelationIntegrityDeclarations, RelationKindRegistration, RelationalSchemaRegistry,
        SchemaId, SchemaRegistryError, SchemaVersionId,
    },
};
use worth_testkit_relational::{
    aspect_storage::definition::{self, aspect_key, field_key},
    durable_medium::TestDurableMedium,
};

/// A Memory runtime whose name aspect is both identity and the policy subject.
pub(super) fn runtime_with_name_merge_policy(
    merge_policy: AspectMergePolicyKind,
) -> RelationalRuntime {
    let name_key = aspect_key("name");
    let registry = register_policy_schema(
        vec![entity_field_aspect(name_key.clone(), field_key("name"))],
        name_key,
        merge_policy,
    )
    .expect("schema registry");
    memory_runtime(registry)
}

/// A Memory runtime with a name identity and one separately declared policy aspect.
pub(super) fn runtime_with_aspect_field_merge_policy(
    aspect_key: AspectKey,
    field_key: FieldKey,
    merge_policy: AspectMergePolicyKind,
) -> RelationalRuntime {
    let registry = register_aspect_field_merge_policy(aspect_key, field_key, merge_policy)
        .expect("schema registry");
    memory_runtime(registry)
}

pub(super) fn register_aspect_field_merge_policy(
    aspect_key: AspectKey,
    field_key: FieldKey,
    merge_policy: AspectMergePolicyKind,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let value_aspect = match merge_policy {
        AspectMergePolicyKind::MonotonicCounter => materialize(
            definition::entity_i64_field_aspect(aspect_key.clone(), field_key),
        ),
        AspectMergePolicyKind::FailOnConflict
        | AspectMergePolicyKind::LastWriterWins
        | AspectMergePolicyKind::AdditiveSet
        | AspectMergePolicyKind::PreferRicher
        | AspectMergePolicyKind::Custom(_) => entity_field_aspect(aspect_key.clone(), field_key),
    };
    register_policy_schema(
        vec![
            entity_field_aspect(
                definition::aspect_key("name"),
                definition::field_key("name"),
            ),
            value_aspect,
        ],
        aspect_key,
        merge_policy,
    )
}

fn register_policy_schema(
    entity_aspects: Vec<DeclaredAspectContractBinding>,
    policy_aspect: AspectKey,
    merge_policy: AspectMergePolicyKind,
) -> Result<RelationalSchemaRegistry, SchemaRegistryError> {
    let name_key = aspect_key("name");
    RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(entity_aspects)
                .with_identity_declarations(vec![IdentityBasisDeclaration {
                    scope: IdentityBasisScope::AspectKey(name_key.clone()),
                    basis: IdentityBasisKind::DeclaredKeySet(vec![name_key].into()),
                }])
                .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                    aspect_key: policy_aspect,
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
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
}

fn memory_runtime(registry: RelationalSchemaRegistry) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .schema_registry(registry)
        .build()
        .expect("valid native runtime configuration")
}

/// The certification-profile durable runtime over label-identified relations.
pub(super) fn durable_runtime_with_topology_identity_registry(
    medium: &Arc<TestDurableMedium>,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(relation_identity_registry())
        .build_durable(medium.attachment())
        .expect("valid native runtime configuration")
}
