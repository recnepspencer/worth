pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::package_root_record::*;

#[test]
fn package_root_frames_are_deterministic_and_reenter_fresh_validation() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let manifest_bytes = encode_manifest_frame(exported.manifest(), limits).unwrap();
    let manifest = decode_manifest_frame(&manifest_bytes, limits).unwrap();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();

    for view in exported.views() {
        let first = encode_package_root_record_frame(view, limits).unwrap();
        let second = encode_package_root_record_frame(view, limits).unwrap();
        assert_eq!(first, second);
        let decoded = decode_package_root_record_frame(&first, limits).unwrap();
        assert_eq!(decoded.canonical_index(), view.canonical_index());
        assert_eq!(decoded.family(), view.family());
        assert_eq!(decoded.record(), view.record());
        let (canonical_index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(canonical_index, record).unwrap();
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
