use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationKindRegistration,
    RelationalSchemaRegistry, SchemaId, SchemaRegistryErrorClass, SchemaVersionId,
};
use crate::facade::schema::{IdentityBasisDeclaration, IdentityBasisKind, IdentityBasisScope};
use crate::tests::support::*;

#[test]
fn schema_identity_declarations_are_defaulted_and_visible_in_trace() {
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
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
                relation_integrity: crate::schema::data::RelationIntegrityDeclarations::default(),
            })
        })
        .unwrap();

    let entity_trace = registry.entity_aspect_declaration_trace(KindId(1)).unwrap();
    let relation_trace = registry
        .relation_aspect_declaration_trace(KindId(2))
        .unwrap();

    assert_eq!(
        entity_trace.identity_declarations,
        vec![
            IdentityBasisDeclaration {
                scope: IdentityBasisScope::EntityKind(KindId(1)),
                basis: IdentityBasisKind::StorageIdentity,
            },
            IdentityBasisDeclaration {
                scope: IdentityBasisScope::EntityKind(KindId(1)),
                basis: IdentityBasisKind::LineageIdentity,
            },
        ]
    );
    assert_eq!(
        relation_trace.identity_declarations,
        vec![IdentityBasisDeclaration {
            scope: IdentityBasisScope::RelationKind(KindId(2)),
            basis: IdentityBasisKind::StorageIdentity,
        }]
    );
}

#[test]
fn relation_schema_rejects_lineage_identity_declarations() {
    let error = RelationalSchemaRegistry::new()
        .register_relation_kind(RelationKindRegistration {
            kind_id: KindId(2),
            kind_name: "test.relation".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            cross_context_policy: CrossContextPolicy::AllowExplicit,
            cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
            aspect_contract_declarations: KindAspectContractDeclarations::default()
                .with_identity_declarations(vec![IdentityBasisDeclaration {
                    scope: IdentityBasisScope::RelationKind(KindId(2)),
                    basis: IdentityBasisKind::LineageIdentity,
                }]),
            relation_integrity: crate::schema::data::RelationIntegrityDeclarations::default(),
        })
        .unwrap_err();

    assert!(matches!(
        error.class,
        SchemaRegistryErrorClass::InvalidAspectDeclaration {
            kind_id: KindId(2),
            ..
        }
    ));
}

#[test]
fn schema_rejects_unsupported_structural_fingerprint_identity_declarations() {
    for basis in [
        IdentityBasisKind::StructuralFingerprint,
        IdentityBasisKind::Custom(crate::facade::schema::CustomIdentityBasisIdentity {
            name: "custom.identity".into(),
            semantic_version: 1,
        }),
    ] {
        let error = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "test.entity".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::default()
                    .with_identity_declarations(vec![IdentityBasisDeclaration {
                        scope: IdentityBasisScope::EntityKind(KindId(1)),
                        basis,
                    }]),
            })
            .unwrap_err();

        assert!(matches!(
            error.class,
            SchemaRegistryErrorClass::InvalidAspectDeclaration {
                kind_id: KindId(1),
                ..
            }
        ));
    }
}
