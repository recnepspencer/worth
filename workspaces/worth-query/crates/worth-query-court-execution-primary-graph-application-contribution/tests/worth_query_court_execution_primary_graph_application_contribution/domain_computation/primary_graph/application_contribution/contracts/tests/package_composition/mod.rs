//! Supported export describes the exact package retained by the installed index.
use worth_query_installation::facade as domain;
use domain::ApplicationSchema;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema as Schema;

fn installed() -> domain::WorthQueryInstalledPackageIndex {
    let package = domain::WorthQueryPortableDomainPackage::new(
        domain::WorthQueryPortableDomainIdentity::new(Schema::OWNER, Schema::MAJOR, Schema::MINOR),
    )
    .application_schema(Schema::declaration().unwrap())
    .validate()
    .unwrap();
    let admitted = domain::WorthQueryInstallationAdmissionProfile::new("export", "configuration")
        .admit(package)
        .unwrap();
    domain::WorthQueryInstalledPackageIndex::build(
        domain::WorthQueryInstallationRuntimeIdentity::fresh(),
        domain::WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
}

#[test]
fn installed_export_reconstructs_and_freshly_validates_against_installed_identity() {
    let installed = installed();
    let expected = installed
        .domain(Schema::OWNER)
        .unwrap()
        .package_identity()
        .clone();
    let exported = installed
        .export_domain_package(
            Schema::OWNER,
            domain::WorthQueryPortablePackageExportLimits::DEFAULT,
        )
        .unwrap();
    assert_eq!(exported.manifest().package_identity(), &expected);
    let mut intake = domain::WorthQueryPortablePackageReconstruction::begin(
        exported.manifest().clone(),
        domain::WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for (index, record) in exported.records().iter().enumerate() {
        intake = intake
            .push_record(u32::try_from(index).unwrap(), record.clone())
            .unwrap();
    }
    let validated = intake
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(
            domain::WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                expected.clone(),
            ),
        )
        .unwrap();
    assert_eq!(validated.identity(), &expected);
}

#[test]
fn export_preserves_typed_unknown_owner_and_actual_byte_refusals() {
    let installed = installed();
    assert_eq!(
        installed
            .export_domain_package(
                "absent",
                domain::WorthQueryPortablePackageExportLimits::DEFAULT,
            )
            .unwrap_err(),
        domain::WorthQueryInstalledPackageExportDenial::DomainNotInstalled {
            owner: "absent".to_owned(),
        }
    );
    let exported = installed
        .export_domain_package(
            Schema::OWNER,
            domain::WorthQueryPortablePackageExportLimits::DEFAULT,
        )
        .unwrap();
    let denial = installed
        .export_domain_package(
            Schema::OWNER,
            domain::WorthQueryPortablePackageExportLimits::new(
                domain::WorthQueryPortablePackageExportLimits::DEFAULT.maximum_records(),
                exported.manifest().logical_export_bytes() - 1,
            ),
        )
        .unwrap_err();
    let domain::WorthQueryInstalledPackageExportDenial::Export(denial) = denial else {
        panic!("installed owner must reach the typed export limit");
    };
    assert_eq!(
        denial.kind(),
        domain::WorthQueryPortablePackageExportDenialKind::LogicalExportBytesExceeded
    );
}
