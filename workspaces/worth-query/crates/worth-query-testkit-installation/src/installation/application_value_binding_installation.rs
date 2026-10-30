pub use worth_foundational::facade::{AspectValue, ScalarAspectType};
pub use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationValueEncodeDenial,
};
pub use worth_query_installation::facade::{
    WorthQueryInstallationAdmissionProfile, WorthQueryInstallationGeneration,
    WorthQueryInstallationRuntimeIdentity, WorthQueryInstalledPackageIndex,
    WorthQueryPortableDomainIdentity, WorthQueryPortableDomainPackage,
};

worth_query_declaration::worth_query_application_schema! {
    pub schema BindingSchema {
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

pub fn installed_index() -> WorthQueryInstalledPackageIndex {
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
