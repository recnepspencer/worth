use worth_foundational::facade::{AspectValue, ScalarAspectType};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationValueEncodeDenial,
};
use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

worth_query_declaration::worth_query_application_schema! {
    schema BindingSchema {
        owner: "worth.tests.value-binding-installation",
        version: (1, 0),
        members: |schema| {
            schema
                .entity(Component::reference())
                .aspect(Component::reference(), Measurement::reference())
                .field(Component::reference(), Count::reference())
        }
    }
}
worth_query_declaration::worth_query_entity!(Component for BindingSchema);
worth_query_declaration::worth_query_aspect!(
    Measurement for BindingSchema, Component;
    identity = AspectIdentity(0x9174_0001),
    revision = AspectContractRevision(1),
);
worth_query_declaration::worth_query_field!(
    Count for BindingSchema, Component, Measurement:
    u64 => worth_query_declaration::facade::application_schema::U64ApplicationValueBinding,
    read_write, equality
);

#[test]
fn installed_schema_retains_exact_typed_scalar_codec_and_contract() {
    let installed = installed_index()
        .bind_application_schema(BindingSchema::declaration().unwrap())
        .unwrap();
    let binding = installed
        .value_bindings()
        .field("Component", "Measurement", "Count")
        .unwrap();

    assert_eq!(installed.value_bindings().len(), 1);
    assert_eq!(binding.schema(), &installed.binding_identity());
    assert_eq!(binding.identity().as_str(), "worth.rust.u64");
    assert_eq!(binding.scalar_family(), ScalarAspectType::UInt64);
    assert!(binding.is_readable());
    assert!(binding.is_identity());
    assert_eq!(binding.encode(&42_u64).unwrap(), AspectValue::UInt64(42));
    assert_eq!(binding.decode::<u64>(&AspectValue::UInt64(7)).unwrap(), 7);

    let denial = binding.encode(&"wrong Rust type").unwrap_err();
    assert!(matches!(
        denial,
        ApplicationValueEncodeDenial::Validation(_)
    ));
    assert_eq!(denial.binding_identity().as_str(), "worth.rust.u64");
}

fn installed_index() -> WorthQueryInstalledPackageIndex {
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        BindingSchema::OWNER,
        1,
        0,
    ))
    .application_schema(BindingSchema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = WorthQueryInstallationAdmissionProfile::new("support", "configuration")
        .admit(package)
        .unwrap();
    WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
}
