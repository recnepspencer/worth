use worth_foundational::AspectKey;

use crate::facade::schema::{AspectMergePolicyDeclaration, AspectMergePolicyKind};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use crate::tests::support::*;

#[test]
fn schema_merge_policy_declarations_are_traced_and_available_through_registry() {
    let name_key = AspectKey::new("name").unwrap();
    let lifecycle_key = AspectKey::new("lifecycle").unwrap();
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(vec![
                entity_field_aspect(name_key.clone(), crate::tests::support::field_key("name")),
                lifecycle_aspect(),
            ])
            .with_merge_policy_declarations(vec![
                AspectMergePolicyDeclaration {
                    aspect_key: name_key.clone(),
                    policy: AspectMergePolicyKind::RetiredValueShape,
                },
                AspectMergePolicyDeclaration {
                    aspect_key: lifecycle_key.clone(),
                    policy: AspectMergePolicyKind::FailOnConflict,
                },
            ]),
        })
        .unwrap();

    let trace = registry.entity_aspect_declaration_trace(KindId(1)).unwrap();
    let policies = registry
        .entity_merge_policy_declarations(KindId(1))
        .unwrap();
    assert_eq!(trace.merge_policy_declarations, policies);
    assert_eq!(
        policies,
        &[
            AspectMergePolicyDeclaration {
                aspect_key: lifecycle_key,
                policy: AspectMergePolicyKind::FailOnConflict,
            },
            AspectMergePolicyDeclaration {
                aspect_key: name_key,
                policy: AspectMergePolicyKind::RetiredValueShape,
            },
        ]
    );
}
