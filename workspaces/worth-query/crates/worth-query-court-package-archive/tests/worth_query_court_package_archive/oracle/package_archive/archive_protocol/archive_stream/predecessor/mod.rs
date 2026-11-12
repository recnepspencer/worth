pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::archive_stream::predecessor::*;

#[test]
fn original_pre_actor_all_family_package_is_unsupported_intact_without_a_candidate() {
    let original = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-query-testkit-package-archive/src/package_archive/archive_protocol/archive_stream/preactor_2bc_all_families.wqpkg"));
    let submitted = original.to_vec();
    // Frozen manifest1 is refused before record decoding or inventory admission.
    let denial = decode_package_archive(&submitted, WorthQueryPackageArchiveLimits::DEFAULT)
        .expect_err("unsupported frame cannot return an archive or reconstruction candidate");
    assert_eq!(
        denial.kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedManifestVersion
    );
    let compatibility = denial.compatibility().unwrap();
    assert_eq!(
        compatibility.layer(),
        WorthQueryPackageArchiveProtocolLayer::Manifest
    );
    assert_eq!(compatibility.observed_version(), 1);
    assert_eq!(u16_at(original, MANIFEST_FRAME_BYTES), 6);
    assert_eq!(WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION, 19);
    assert_eq!(submitted, original.as_slice());
}
