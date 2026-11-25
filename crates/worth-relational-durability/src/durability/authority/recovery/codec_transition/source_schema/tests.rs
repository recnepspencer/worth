//! Root-free admission over the actual installed program roster.
use super::*;
use contracts::config::data::policies::{CascadeDeletePolicy, CrossContextPolicy};
use contracts::identity::data::primitives::KindId;
use contracts::schema::data::relation_integrity::declarations::RelationIntegrityDeclarations;
use contracts::schema::data::{
    EntityKindRegistration, RelationKindRegistration, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use worth_foundational::facade::{AspectBinding, AspectKey, FieldKey};
use worth_relational_contracts::internal_port as contracts;

#[test]
fn source_only_codec_consequences_refuse_affected_support_and_allow_an_unrelated_kind() {
    let (mut registry, program) = managed_cache_schema();
    let unrelated_kind = KindId(
        (*registry.owner_entity_kinds())
            .keys()
            .chain((*registry.owner_relation_kinds()).keys())
            .map(|kind| kind.0)
            .max()
            .unwrap()
            + 1,
    );
    let schema = (*registry.owner_entity_kinds())[&KindId(1)]
        .schema_id
        .clone();
    registry = registry
        .register_entity_kind(contracts::schema::data::EntityKindRegistration {
            kind_id: unrelated_kind,
            kind_name: "outside-managed-source".into(),
            schema_id: schema,
            schema_version_id: contracts::schema::data::SchemaVersionId(1),
            aspect_contract_declarations: Default::default(),
        })
        .unwrap();
    let installed = FrozenCustomInvariantRegistry::compile(
        &registry,
        vec![],
        vec![program],
        contracts::schema::compilation_control::SchemaCompilationControl::Configuration,
    )
    .unwrap();
    assert_eq!(installed.len(), 0, "no invariant supplies this consequence");
    let change = |kind_id| CodecFieldLocus {
        kind_id,
        aspect_key: AspectKey::new("codec-change").unwrap(),
        field_key: FieldKey::new("value").unwrap(),
        binding: AspectBinding::EntityField {
            field: FieldKey::new("value").unwrap(),
        },
    };
    assert_eq!(
        admit_supported_consequences(&installed, [], &[change(KindId(1))]),
        Err("codec transition with managed Source consequences is unsupported".into()),
    );
    assert_eq!(
        admit_supported_consequences(&installed, [], &[change(unrelated_kind)]),
        Ok(()),
    );
}

use contracts::derived_relation::definition::{
    DerivedRelationDefinitionParts, DerivedRelationIdentity, DerivedRelationNode,
};
use contracts::validation::candidate_source::program::ManagedIndexSourceProgram;
fn managed_cache_schema() -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    let identity = DerivedRelationIdentity {
        schema: SchemaId("test".into()),
        version: SchemaVersionId(1),
        name: "managed-index-source::cache-court".into(),
    };
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "managed-source".into(),
            schema_id: identity.schema.clone(),
            schema_version_id: identity.version,
            aspect_contract_declarations: Default::default(),
        })
        .unwrap()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(3),
            kind_name: "cache-court-membership".into(),
            schema_id: SchemaId("test".into()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::Forbid,
            cascade_delete_policy: CascadeDeletePolicy::RetainDanglingForAudit,
            aspect_contract_declarations: Default::default(),
            relation_integrity: RelationIntegrityDeclarations::new(
                vec![contracts::schema::data::relation_integrity::declarations::EndpointKindContractDeclaration {
                    contract_id: "cache-court-endpoints".into(),
                    allowed_source_kinds: vec![KindId(1)],
                    allowed_target_kinds: vec![KindId(1)],
                    self_edges_allowed: false,
                    cross_context_policy: CrossContextPolicy::Forbid,
                }],
                vec![],
                vec![],
                vec![],
                vec![],
            ),
        })
        .unwrap()
        .register_derived_relations(vec![DerivedRelationDefinitionParts {
            identity: identity.clone(),
            from: KindId(1).into(),
            to: KindId(1).into(),
            nodes: vec![DerivedRelationNode::Stored {
                relation: KindId(3),
                from: KindId(1),
                to: KindId(1),
            }],
            laws: None,
        }])
        .unwrap();
    let source = registry
        .bind_managed_index_source(
            &registry.derived_relation(&identity).unwrap(),
            vec![],
            vec![],
        )
        .unwrap();
    (registry, source)
}
