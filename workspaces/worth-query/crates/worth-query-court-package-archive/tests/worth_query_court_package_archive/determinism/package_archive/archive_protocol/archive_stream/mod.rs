pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::archive_stream::*;

#[test]
fn version_two_minimal_archive_is_deterministic_and_frozen() {
    // package/identity.rs:81 declares canonical identity v4: hash the fixture's
    // canonical tokens with that domain, then encode the v2 archive framing.
    let export = fixture::minimal_package().export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let first = encode_package_archive(&export, limits).unwrap();
    assert_eq!(encode_package_archive(&export, limits).unwrap(), first);
    assert_eq!(encode_hex(&first), VERSION_TWO_MINIMAL_ARCHIVE_HEX.trim());
    let frozen = decode_hex(VERSION_TWO_MINIMAL_ARCHIVE_HEX.trim());
    assert_eq!(u16_at(&frozen, 8), 2, "outer archive version stays2");
    assert_eq!(u16_at(&frozen, MANIFEST_FRAME_BYTES), 19);
    let decoded = decode_package_archive(&frozen, limits).unwrap();
    assert_eq!(decoded.manifest(), export.manifest());
    assert_eq!(decoded.frames().len(), 1);
    assert_eq!(decoded.frames()[0].record(), &export.records()[0]);
}
