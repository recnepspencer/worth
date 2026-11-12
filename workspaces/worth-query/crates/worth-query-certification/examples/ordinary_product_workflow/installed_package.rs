//! Inspect retained installed meaning and use the existing SDK readmission path.
use super::product_workflow_support::{schema::TemporalHostSchema, ExampleApplication};
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_host::facade::domain;

pub(super) fn verify(application: &ExampleApplication) {
    let runtime = application.runtime.runtime();
    let installed = runtime.installed_schema().binding_identity();
    let expected = domain::WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes(
        *installed.package_identity().bytes(),
    );
    let records = runtime
        .export_domain_package(
            TemporalHostSchema::OWNER,
            domain::WorthQueryPortablePackageExportLimits::DEFAULT,
        )
        .expect("export the package actually retained by this installation");
    let mut intake = domain::WorthQueryPortablePackageReconstruction::begin(
        records.manifest().clone(),
        domain::WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .expect("begin bounded SDK record reconstruction");
    for (index, record) in records.records().iter().enumerate() {
        intake = intake
            .push_record(u32::try_from(index).unwrap(), record.clone())
            .expect("the exported record sequence is complete");
    }
    let validated = intake
        .close()
        .expect("close the record sequence")
        .materialize()
        .expect("materialize descriptive package meaning")
        .validate_freshly(
            domain::WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                expected.clone(),
            ),
        )
        .expect("fresh validation matches independently obtained installed identity");
    assert_eq!(validated.identity(), &expected);
}
