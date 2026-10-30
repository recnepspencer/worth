pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::manifest_frame::*;

#[test]
fn manifest_frame_is_deterministic() {
    let first =
        encode_manifest_frame(&manifest(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let second =
        encode_manifest_frame(&manifest(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    assert_eq!(first, second);
    let decoded = decode_manifest_frame(&first, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    assert_eq!(decoded.package_identity().bytes(), &[0x41; 32]);
    WorthQueryPortablePackageReconstruction::begin(
        decoded,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
}
