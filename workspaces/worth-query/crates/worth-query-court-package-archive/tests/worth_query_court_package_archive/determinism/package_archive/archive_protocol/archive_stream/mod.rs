pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::archive_stream::*;

#[test]
fn version_two_minimal_archive_is_deterministic_and_frozen() {
    for (version, vector) in [
        (6_u16, RETAINED_VERSION_TWO_MINIMAL_ARCHIVE_HEX),
        (7_u16, RETAINED_RECORD_SEVEN_MINIMAL_ARCHIVE_HEX),
        (8_u16, RETAINED_RECORD_EIGHT_MINIMAL_ARCHIVE_HEX),
        (9_u16, RETAINED_RECORD_NINE_MINIMAL_ARCHIVE_HEX),
        (10_u16, RETAINED_RECORD_TEN_MINIMAL_ARCHIVE_HEX),
        (12_u16, RETAINED_RECORD_TWELVE_MINIMAL_ARCHIVE_HEX),
        (13_u16, RETAINED_RECORD_THIRTEEN_MINIMAL_ARCHIVE_HEX),
        (14_u16, RETAINED_RECORD_FOURTEEN_MINIMAL_ARCHIVE_HEX),
        (16_u16, RETAINED_RECORD_SIXTEEN_MINIMAL_ARCHIVE_HEX),
        (17_u16, RETAINED_RECORD_SEVENTEEN_MINIMAL_ARCHIVE_HEX),
    ] {
        let original = decode_hex(vector.trim());
        let denial =
            decode_package_archive(&original, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err();
        assert_eq!(
            denial.kind(),
            WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
        );
        assert_eq!(denial.compatibility().unwrap().observed_version(), version);
    }

    // package/identity.rs:81 declares canonical identity v4: hash the fixture's
    // canonical tokens with that domain, then encode the v2 archive framing.
    let export = fixture::minimal_package().export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let first = encode_package_archive(&export, limits).unwrap();
    assert_eq!(encode_package_archive(&export, limits).unwrap(), first);
    assert_eq!(encode_hex(&first), VERSION_TWO_MINIMAL_ARCHIVE_HEX.trim());
    let frozen = decode_hex(VERSION_TWO_MINIMAL_ARCHIVE_HEX.trim());
    assert_eq!(u16_at(&frozen, 8), 2, "outer archive version stays2");
    assert_eq!(u16_at(&frozen, MANIFEST_FRAME_BYTES), 18);
    let decoded = decode_package_archive(&frozen, limits).unwrap();
    assert_eq!(decoded.manifest(), export.manifest());
    assert_eq!(decoded.frames().len(), 1);
    assert_eq!(decoded.frames()[0].record(), &export.records()[0]);
}
