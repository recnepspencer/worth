//! Private schema reuse predicate over immutable values, never a runtime world.
use super::*;
use crate::schema::data::{
    DescriptorSemanticsVersion, EntityKindRegistration, KindAspectContractDeclarations,
    RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};

#[test]
fn equal_schema_digest_reuses_only_equal_descriptor_semantics() {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: crate::identity::data::KindId(1),
            kind_name: "branch.schema.reuse".into(),
            schema_id: SchemaId("branch.schema".into()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::default(),
        })
        .unwrap();
    let schema = registry.authority_snapshot();
    let original =
        crate::schema::data::runtime_descriptor_semantics_policy().current_write_version();
    let previous = RelationalBranchRoot::empty_with_schema(&registry, original);
    let issuer = RelationalBranchRootIdentityIssuer::default();
    issuer.observe_root(&previous);
    let same =
        prepare_schema(&issuer, Some(&previous), &schema, original, &registry, None).unwrap();
    assert!(Arc::ptr_eq(&same.authority, &previous.schema_authority));

    // Isolate the independent descriptor-format input to this private predicate.
    // This proves allocation reuse, not admission of a future runtime format.
    let successor = DescriptorSemanticsVersion(original.0.checked_add(1).unwrap());
    assert_eq!(
        schema.digest(),
        previous.schema_authority.authority_digest()
    );
    let changed = prepare_schema(
        &issuer,
        Some(&previous),
        &schema,
        successor,
        &registry,
        None,
    )
    .unwrap();
    assert_eq!(changed.schema_root, same.schema_root);
    assert_eq!(changed.authority.descriptor_semantics_version(), successor);
    assert!(!Arc::ptr_eq(&changed.authority, &previous.schema_authority));
    assert_eq!(
        previous.schema_authority.descriptor_semantics_version(),
        original
    );
    assert_eq!(changed.authority.authority_digest(), schema.digest());
}
