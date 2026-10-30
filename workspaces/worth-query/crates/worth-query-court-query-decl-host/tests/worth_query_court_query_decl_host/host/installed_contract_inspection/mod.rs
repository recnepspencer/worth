pub(crate) use worth_query_testkit_host::facade::host::installed_contract_inspection::*;

#[test]
fn host_consumer_inspects_one_exact_typed_installed_contract() {
    let declaration = ContractInspectionSchema::declaration()
        .expect("the public declaration facade should build the schema");
    let package = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        "contract_inspection",
        1,
        0,
    ))
    .application_schema(declaration.clone())
    .validate()
    .expect("the public package should validate");
    portable_export::assert_complete_typed_export(&package);
    portable_spine::assert_portable_contract_spine(&package);
    let admitted = WorthQueryInstallationAdmissionProfile::new("host", "contract-inspection")
        .admit(package)
        .expect("the public package should admit");
    let index = worth_query_host::facade::domain::WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .expect("the public package should install");
    let schema = index
        .bind_application_schema(declaration)
        .expect("the typed schema should bind");

    assert_native_catalog(&schema);
    assert_mutation_read_contract(&schema);
    assert_mutation_touch_contract(&schema);
    assert_emit_only_aftermath_contract(&schema);
}
