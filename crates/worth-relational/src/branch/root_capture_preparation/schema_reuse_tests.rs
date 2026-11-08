//! Private schema reuse predicate over immutable values, never a runtime world.
use super::*;
use crate::schema::data::DescriptorSemanticsVersion;
use crate::tests::support::{canonical_schema_envelope_input, AspectSchemaFixture};

#[test]
fn equal_schema_digest_reuses_only_equal_descriptor_semantics() {
    let registry = AspectSchemaFixture::with_default_declared_aspects(
        crate::config::data::CascadeDeletePolicy::CascadeDeleteRelations,
    )
    .build_registry();
    let mut envelope = canonical_schema_envelope_input(&registry, None);
    let previous =
        RelationalBranchRoot::empty_with_schema(&registry, envelope.descriptor_semantics_version);
    let issuer = RelationalBranchRootIdentityIssuer::default();
    issuer.observe_root(&previous);
    let same = prepare_schema(&issuer, Some(&previous), &envelope, &registry).unwrap();
    assert!(Arc::ptr_eq(&same.authority, &previous.schema_authority));

    // Isolate the independent descriptor-format input to this private predicate.
    // This proves allocation reuse, not admission of a future runtime format.
    let original = envelope.descriptor_semantics_version;
    envelope.descriptor_semantics_version =
        DescriptorSemanticsVersion(original.0.checked_add(1).unwrap());
    assert_eq!(
        envelope.schema_authority.digest(),
        previous.schema_authority.authority_digest()
    );
    let changed = prepare_schema(&issuer, Some(&previous), &envelope, &registry).unwrap();
    assert_eq!(changed.schema_root, same.schema_root);
    assert_eq!(
        changed.authority.descriptor_semantics_version(),
        envelope.descriptor_semantics_version
    );
    assert!(!Arc::ptr_eq(&changed.authority, &previous.schema_authority));
    assert_eq!(
        previous.schema_authority.descriptor_semantics_version(),
        original
    );
    assert_eq!(
        changed.authority.snapshot().digest(),
        envelope.schema_authority.digest()
    );
}
