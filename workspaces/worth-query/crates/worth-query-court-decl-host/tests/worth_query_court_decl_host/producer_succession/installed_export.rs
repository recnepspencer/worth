//! Export traverses the real installed program's composed contribution package.
use super::{world, ApplicationSchema, Schema};
use std::sync::Arc;
use worth_query_host::facade::domain;

#[test]
fn installed_composed_package_exports_and_reconstructs_with_headroom() {
    let host = world::install(2, Arc::default());
    let runtime = host.runtime();
    // Expectation comes from installed schema authority, independently of intake.
    let installed_identity = runtime.installed_schema().binding_identity();
    let expected = domain::WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes(
        *installed_identity.package_identity().bytes(),
    );
    let exported = runtime
        .export_domain_package(
            Schema::OWNER,
            domain::WorthQueryPortablePackageExportLimits::DEFAULT,
        )
        .unwrap();
    assert_eq!(
        runtime
            .export_domain_package(
                "absent",
                domain::WorthQueryPortablePackageExportLimits::DEFAULT,
            )
            .unwrap_err(),
        domain::WorthQueryInstalledPackageExportDenial::DomainNotInstalled {
            owner: "absent".to_owned(),
        }
    );
    assert_eq!(exported.manifest().package_identity(), &expected);
    for family in [
        domain::WorthQueryPortablePackageRecordFamily::DomainOperation,
        domain::WorthQueryPortablePackageRecordFamily::ConditionalApplicationOperation,
    ] {
        assert!(
            exported.manifest().family_count(family) > 0,
            "production composition must be present in exported meaning"
        );
    }
    let defaults = domain::WorthQueryPortablePackageReconstructionLimits::DEFAULT;
    let limits = defaults.with_work_bounds(
        defaults.maximum_logical_bytes(),
        defaults.maximum_nested_entries(),
        56 * 1024 * 1024,
    );
    let mut intake =
        domain::WorthQueryPortablePackageReconstruction::begin(exported.manifest().clone(), limits)
            .unwrap();
    for (index, record) in exported.records().iter().enumerate() {
        intake = intake
            .push_record(u32::try_from(index).unwrap(), record.clone())
            .unwrap();
    }
    let candidate = intake.close().unwrap().materialize().unwrap();
    eprintln!(
        "installed composed export: records={} logical_bytes={} materialized_canonical_bytes={}",
        exported.records().len(),
        exported.manifest().logical_export_bytes(),
        candidate.work().canonical_work_bytes()
    );
    let validated = candidate
        .validate_freshly(
            domain::WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                expected.clone(),
            ),
        )
        .unwrap();
    assert_eq!(validated.identity(), &expected);
    let required = exported.manifest().canonical_source_bytes();
    assert!(required > 0);
    let narrowed = limits.with_work_bounds(
        defaults.maximum_logical_bytes(),
        defaults.maximum_nested_entries(),
        required - 1,
    );
    assert!(
        matches!(domain::WorthQueryPortablePackageReconstruction::begin(
        exported.manifest().clone(), narrowed,
    ), Err(domain::WorthQueryPortablePackageReconstructionDenial::DeclaredCanonicalWorkBudgetExceeded {
        declared, maximum,
    }) if declared == required && maximum == required - 1)
    );
}
