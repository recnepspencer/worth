use worth_foundational::AspectKey;

use crate::facade::schema::{
    AspectMergePolicyDeclaration, AspectMergePolicyKind, IdentityBasisDeclaration,
    IdentityBasisKind, IdentityBasisScope,
};
use crate::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationalSchemaRegistry, SchemaId,
    SchemaVersionId,
};
use crate::tests::support::*;

#[test]
fn schema_plan_revision_changes_when_identity_or_merge_policy_semantics_change() {
    let name_key = AspectKey::new("name").unwrap();
    let base = KindAspectContractDeclarations::new(vec![entity_field_aspect(
        name_key.clone(),
        crate::tests::support::field_key("name"),
    )]);
    let identity_variant = base.clone().with_identity_declarations(vec![
        IdentityBasisDeclaration {
            scope: IdentityBasisScope::EntityKind(KindId(1)),
            basis: IdentityBasisKind::StorageIdentity,
        },
        IdentityBasisDeclaration {
            scope: IdentityBasisScope::AspectKey(name_key.clone()),
            basis: IdentityBasisKind::DeclaredKeySet(vec![name_key.clone()].into()),
        },
    ]);
    let policy_variant =
        base.clone()
            .with_merge_policy_declarations(vec![AspectMergePolicyDeclaration {
                aspect_key: name_key.clone(),
                policy: AspectMergePolicyKind::RetiredValueShape,
            }]);

    let base_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: base,
        })
        .unwrap();
    let identity_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: identity_variant,
        })
        .unwrap();
    let policy_registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: KindId(1),
            kind_name: "test.entity".to_string(),
            schema_id: SchemaId("test".to_string()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: policy_variant,
        })
        .unwrap();

    let base_revision = base_registry
        .entity_registration(KindId(1))
        .unwrap()
        .aspect_contract_declarations
        .plan_revision;
    let identity_revision = identity_registry
        .entity_registration(KindId(1))
        .unwrap()
        .aspect_contract_declarations
        .plan_revision;
    let policy_revision = policy_registry
        .entity_registration(KindId(1))
        .unwrap()
        .aspect_contract_declarations
        .plan_revision;

    assert_ne!(base_revision, identity_revision);
    assert_ne!(base_revision, policy_revision);
    assert_ne!(identity_revision, policy_revision);
}
