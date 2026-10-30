pub use super::*;
pub use worth_query_host::facade::domain::{
    WorthQueryPortablePackageRecord, WorthQueryPortablePackageRecordFamily,
    WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
};

pub fn assert_complete_typed_export(package: &WorthQueryValidatedPortableDomainPackage) {
    let export = package
        .export_typed_records()
        .expect("a freshly validated package exports under the default limits");
    let manifest = export.manifest();
    assert_eq!(
        manifest.version(),
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION
    );
    assert_eq!(manifest.package_identity(), package.identity());
    assert_eq!(manifest.record_count(), 6);
    assert!(manifest.canonical_source_bytes() > 0);
    assert!(manifest.logical_export_bytes() > manifest.canonical_source_bytes());

    for (family, expected) in [
        (WorthQueryPortablePackageRecordFamily::DomainIdentity, 1),
        (WorthQueryPortablePackageRecordFamily::ApplicationSchema, 1),
        (
            WorthQueryPortablePackageRecordFamily::NativeAspectContract,
            2,
        ),
        (
            WorthQueryPortablePackageRecordFamily::ApplicationOperationContract,
            2,
        ),
    ] {
        assert_eq!(manifest.family_count(family), expected);
    }
    assert!(export
        .records()
        .windows(2)
        .all(|pair| pair[0].family() <= pair[1].family()));

    let exported_native = export.records().iter().filter_map(|record| match record {
        WorthQueryPortablePackageRecord::NativeAspectContract(contract) => Some(contract),
        _ => None,
    });
    assert_eq!(
        exported_native.collect::<Vec<_>>(),
        package
            .application_contract_spine()
            .native_aspects()
            .iter()
            .collect::<Vec<_>>()
    );
    let exported_operations = export.records().iter().filter_map(|record| match record {
        WorthQueryPortablePackageRecord::ApplicationOperationContract(contract) => Some(contract),
        _ => None,
    });
    assert_eq!(
        exported_operations.collect::<Vec<_>>(),
        package
            .application_contract_spine()
            .operations()
            .iter()
            .collect::<Vec<_>>()
    );
}
