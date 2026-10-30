//! Actual installed binding twins; portable names cannot substitute for their owners.
use super::*;
use crate::application_principal_binding_tests::{
    IdentitySchema, Principal, PrincipalQueryBinding,
};
use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_query_declaration::facade::application_query::{
    ApplicationManagedIndexBinding, ApplicationQueryBindingReference,
};
use worth_query_declaration::facade::application_schema::ValueDecodeBounds;
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(worth_foundational::facade::WorthValue)]
pub struct Probe {
    prefix: String,
}
worth_query_declaration::worth_query_value_binding!(pub ProbeBinding for Probe {
    identity: "worth.query.tests.index-probe.v1", scalar: String,
    canonical: ValueDecodeBounds::new(1024, 512, 4, 4),
});
struct Index;
impl ApplicationManagedIndexBinding<IdentitySchema> for Index {
    type SourceBinding = PrincipalQueryBinding;
    type ProbeBinding = ProbeBinding;
    type Entity = Principal;
    const NAME: &'static str = "PrincipalPrefix";
    const PROVIDER: &'static str = "worth.query.tests.prefix-provider.v1";
    const PROVIDER_VERSION: u32 = 1;
}

#[test]
fn managed_index_checks_actual_binding_and_probe_before_installation() {
    let declaration = IdentitySchema::declaration().unwrap();
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        IdentitySchema::OWNER,
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    let index = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap();
    let schema = index.bind_application_schema(declaration).unwrap();
    let recipe = Index::descriptor();
    assert!(recipe.matches::<IdentitySchema, Index>());
    let source = schema
        .query_catalog
        .get_binding(recipe.source().identity())
        .unwrap();
    let bindings = BTreeMap::from([(recipe.source().identity().to_owned(), source)]);
    let valid = ApplicationSchemaMember::ManagedIndex {
        definition: recipe.meaning().clone(),
    };
    validate_definitions(
        std::slice::from_ref(&valid),
        std::slice::from_ref(&recipe),
        &[],
        &bindings,
    )
    .unwrap();
    assert_eq!(
        validate_definitions(std::slice::from_ref(&valid), &[], &[], &bindings)
            .unwrap_err()
            .kind(),
        Kind::ManagedIndexBindingMeaningChanged
    );
    let mut parts = recipe.meaning().parts().clone();
    let mut changed = parts.source.binding.parts().clone();
    changed.input = WorthQueryPortableTypeIdentity::declared("worth.query.tests.foreign-input.v1");
    parts.source.binding = ApplicationQueryBindingReference::from_untrusted_parts(changed);
    let foreign = ApplicationSchemaMember::ManagedIndex {
        definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(parts),
    };
    assert_eq!(
        validate_definitions(&[foreign], std::slice::from_ref(&recipe), &[], &bindings)
            .unwrap_err()
            .kind(),
        Kind::ManagedIndexBindingMeaningChanged
    );
    let mut parts = recipe.meaning().parts().clone();
    parts.probe =
        worth_query_declaration::facade::application_query::ApplicationManagedIndexProbeMeaning::of::<
            worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
        >();
    let invalid = ApplicationSchemaMember::ManagedIndex {
        definition: ErasedApplicationManagedIndexDefinition::from_untrusted_parts(parts),
    };
    assert_eq!(
        validate_definitions(&[invalid], &[recipe], &[], &bindings)
            .unwrap_err()
            .kind(),
        Kind::InvalidManagedIndexDefinition
    );
}
