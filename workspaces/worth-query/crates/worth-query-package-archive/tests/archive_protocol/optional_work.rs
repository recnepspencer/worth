//! Actual portable domain export/readmission with an omitted aggregate budget.
use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits,
};
use worth_query_package_archive::facade::*;

#[test]
fn omitted_work_budget_roundtrips_an_ordinary_domain_package() {
    let source = super::domain_operation_record::fixture::operation_package_without_work_budget();
    let bounded = super::domain_operation_record::fixture::operation_package();
    assert_ne!(source.identity(), bounded.identity());
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    for view in exported.views() {
        let bytes = encode_record_frame(view, limits).unwrap();
        let decoded = decoder.decode_frame(&bytes).unwrap();
        assert_eq!(decoded.record(), view.record());
        let (index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }
    let reconstructed = reconstruction
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .unwrap();
    assert_eq!(reconstructed.identity(), source.identity());
}
