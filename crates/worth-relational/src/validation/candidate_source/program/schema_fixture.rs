//! The finite declarations used by Source binding tests.
use super::ManagedIndexSourceProgram;
use crate::config::data::{CascadeDeletePolicy, CrossContextPolicy};
use crate::derived_relation::definition::{
    DerivedRelationDefinitionParts, DerivedRelationIdentity, DerivedRelationNode,
};
use crate::identity::data::KindId;
use crate::schema::data::{
    EndpointKindContractDeclaration, EntityKindRegistration, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};

pub(crate) fn managed_cache_schema() -> (RelationalSchemaRegistry, ManagedIndexSourceProgram) {
    let identity = DerivedRelationIdentity {
        schema: SchemaId("test".into()),
        version: SchemaVersionId(1),
        name: "managed-index-source::cache-court".into(),
    };
    let registry = RelationalSchemaRegistry::default()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "cache-court-entity".into(),
            schema_id: SchemaId("test".into()),
            schema_version_id: SchemaVersionId(1),
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
                vec![EndpointKindContractDeclaration {
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
