use worth_foundational::AspectKey;

use crate::facade::schema::{AspectMergePolicyDeclaration, AspectMergePolicyKind};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaRegistryErrorClass, SchemaVersionId,
};
use crate::tests::support::*;

#[test]
fn schema_accepts_runtime_owned_record_field_merge_policies_and_rejects_unsupported_policies() {
    let name_key = AspectKey::new("name").unwrap();
    let counter_key = AspectKey::new("counter").unwrap();
    let accepted = [
        (
            name_key.clone(),
            entity_field_aspect(name_key.clone(), crate::tests::support::field_key("name")),
            AspectMergePolicyKind::RetiredWriterOrder,
        ),
        (
            counter_key.clone(),
            entity_i64_field_aspect(
                counter_key.clone(),
                crate::tests::support::field_key("counter"),
            ),
            AspectMergePolicyKind::MonotonicCounter,
        ),
    ];

    for (aspect_key, declared_aspect, policy) in accepted {
        let registry = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "test.entity".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    declared_aspect,
                ])
                .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                    aspect_key: aspect_key.clone(),
                    policy: policy.clone(),
                }]),
            })
            .expect("runtime-owned record field policy should register");

        assert_eq!(
            registry
                .entity_merge_policy_declarations(KindId(1))
                .unwrap(),
            &[AspectMergePolicyDeclaration { aspect_key, policy }]
        );
    }

    for rejected_policy in [
        AspectMergePolicyKind::AdditiveSet,
        AspectMergePolicyKind::Custom(crate::facade::schema::CustomMergePolicyIdentity {
            name: "custom.merge".into(),
            semantic_version: 1,
        }),
    ] {
        let error = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "test.entity".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    entity_field_aspect(name_key.clone(), crate::tests::support::field_key("name")),
                ])
                .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                    aspect_key: name_key.clone(),
                    policy: rejected_policy,
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

#[test]
fn schema_rejects_monotonic_counter_on_non_integer_foundational_contract() {
    let name_key = AspectKey::new("name").unwrap();
    let error = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(name_key.clone(), crate::tests::support::field_key("name")),
            ])
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: name_key,
                policy: AspectMergePolicyKind::MonotonicCounter,
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

#[test]
fn schema_rejects_runtime_owned_merge_policies_on_non_record_field_aspects() {
    let lifecycle_key = AspectKey::new("lifecycle").unwrap();
    for policy in [
        AspectMergePolicyKind::Custom(crate::facade::schema::CustomMergePolicyIdentity {
            name: "custom.merge".into(),
            semantic_version: 1,
        }),
        AspectMergePolicyKind::RetiredWriterOrder,
        AspectMergePolicyKind::MonotonicCounter,
        AspectMergePolicyKind::AdditiveSet,
    ] {
        let error = RelationalSchemaRegistry::new()
            .register_entity_kind(EntityKindRegistration {
                kind_id: KindId(1),
                kind_name: "test.entity".to_string(),
                schema_id: SchemaId("test".to_string()),
                schema_version_id: SchemaVersionId(1),
                aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                    lifecycle_aspect(),
                ])
                .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                    aspect_key: lifecycle_key.clone(),
                    policy,
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

#[test]
fn schema_rejects_merge_policy_for_undeclared_aspect_key() {
    let error = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default()
                .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                    aspect_key: AspectKey::new("missing").unwrap(),
                    policy: AspectMergePolicyKind::FailOnConflict,
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
