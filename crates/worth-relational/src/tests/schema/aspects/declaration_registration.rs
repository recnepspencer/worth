use worth_foundational::AspectKey;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;

use crate::facade::schema::{
    DeclaredAspectContractBinding, EntityKindRegistration, KindAspectContractDeclarations,
    RelationalSchemaRegistry, SchemaId, SchemaRegistryErrorClass, SchemaVersionId,
};
use crate::tests::schema::aspects::fixtures::lifecycle_aspect_named;
use crate::tests::support::*;

#[test]
fn schema_aspect_declarations_are_canonicalized_before_revision_is_derived() {
    let first = KindAspectContractDeclarations::new(vec![
        lifecycle_aspect_named(crate::tests::support::aspect_key("zeta")),
        entity_field_aspect(
            crate::tests::support::aspect_key("alpha"),
            crate::tests::support::field_key("name"),
        ),
    ]);
    let second = KindAspectContractDeclarations::new(vec![
        entity_field_aspect(
            crate::tests::support::aspect_key("alpha"),
            crate::tests::support::field_key("name"),
        ),
        lifecycle_aspect_named(crate::tests::support::aspect_key("zeta")),
    ]);

    let first_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: first,
        })
        .unwrap();
    let second_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: second,
        })
        .unwrap();

    let left = (*first_registry.owner_entity_kinds())
        .get(&KindId(1))
        .unwrap();
    let right = (*second_registry.owner_entity_kinds())
        .get(&KindId(1))
        .unwrap();

    assert_eq!(
        left.aspect_contract_declarations.plan_revision,
        right.aspect_contract_declarations.plan_revision
    );
    assert_eq!(
        left.aspect_contract_declarations
            .aspects
            .iter()
            .map(DeclaredAspectContractBinding::aspect_key)
            .collect::<Vec<_>>(),
        right
            .aspect_contract_declarations
            .aspects
            .iter()
            .map(DeclaredAspectContractBinding::aspect_key)
            .collect::<Vec<_>>()
    );
}

#[test]
fn duplicate_aspect_keys_are_rejected() {
    let error = RelationalSchemaRegistry::new()
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
                lifecycle_aspect_named(crate::tests::support::aspect_key("name")),
            ]),
        })
        .unwrap_err();

    assert!(matches!(
        error.class,
        SchemaRegistryErrorClass::DuplicateAspectKey {
            kind_id: KindId(1),
            ..
        }
    ));
}

#[test]
fn declared_aspect_contract_keys_round_trip_through_registration() {
    let registry = RelationalSchemaRegistry::new()
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
            ]),
        })
        .expect("registry");

    let trace = registry.entity_aspect_declaration_trace(KindId(1)).unwrap();
    assert_eq!(
        trace.declarations[0].aspect_key,
        AspectKey::new("name").unwrap()
    );
    assert_eq!(trace.declarations[0].contract_revision, 1);
}
