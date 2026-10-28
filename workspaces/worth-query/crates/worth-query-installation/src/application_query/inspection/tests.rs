use super::*;
use crate::application_principal_binding_tests::{IdentitySchema, PrincipalQueryBinding};
use crate::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBindingReference, ApplicationQueryDisclosureContract,
    ApplicationQueryInspectionDefinition, ErasedApplicationQueryInspectionDefinition,
};
use worth_query_declaration::facade::portable_identity::WorthQueryPortableTypeIdentity;

#[test]
fn inspection_resolves_every_definition_against_actual_compiled_binding_before_unavailable() {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "identity_installation_test",
        1,
        0,
    ))
    .application_schema(IdentitySchema::declaration().unwrap())
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
    let schema = index
        .bind_application_schema(IdentitySchema::declaration().unwrap())
        .unwrap();
    let definition =
        ApplicationQueryInspectionDefinition::<IdentitySchema, PrincipalQueryBinding>::new(
            ApplicationQueryDisclosureContract::public(),
        )
        .unwrap()
        .into_erased();
    let reference = &definition.parts().binding;
    let name = reference.parts().binding.as_str();
    let bindings = BTreeMap::from([(
        name.to_owned(),
        schema.query_catalog.get_binding(name).unwrap(),
    )]);
    let valid = ApplicationSchemaMember::ApplicationQueryInspection {
        definition: definition.clone(),
    };
    assert_eq!(
        validate_definitions(&[valid.clone()], &bindings)
            .unwrap_err()
            .kind(),
        Kind::InspectionUnavailable
    );
    // The installed descriptor is genuine; only the untrusted descriptive input
    // changes. Matching the binding's display name would accept this counterfeit.
    let mut parts = definition.parts().clone();
    let mut changed = parts.binding.parts().clone();
    changed.input = WorthQueryPortableTypeIdentity::declared("worth.query.foreign-input.v1");
    parts.binding = ApplicationQueryBindingReference::from_untrusted_parts(changed);
    let foreign = ApplicationSchemaMember::ApplicationQueryInspection {
        definition: ErasedApplicationQueryInspectionDefinition::from_untrusted_parts(parts),
    };
    assert_eq!(
        validate_definitions(&[foreign], &bindings)
            .unwrap_err()
            .kind(),
        Kind::InspectionBindingMeaningChanged
    );
    let mut parts = definition.parts().clone();
    let mut missing = parts.binding.parts().clone();
    missing.binding =
        WorthQueryPortableTypeIdentity::declared("worth.query.uninstalled-binding.v1");
    parts.binding = ApplicationQueryBindingReference::from_untrusted_parts(missing);
    let missing = ApplicationSchemaMember::ApplicationQueryInspection {
        definition: ErasedApplicationQueryInspectionDefinition::from_untrusted_parts(parts),
    };
    assert_eq!(
        validate_definitions(&[valid, missing], &bindings)
            .unwrap_err()
            .kind(),
        Kind::InspectionBindingMeaningChanged
    );
}
