pub(crate) use worth_query_worlds::facade::package_archive::archive_protocol::archive_stream::predecessor::*;

#[test]
fn retained_version_fifteen_package_is_refused_intact() {
    let bytes = super::decode_hex(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-query-worlds/src/package_archive/archive_protocol/archive_stream/archive_v2_record_v15.hex"))
            .trim(),
    );
    let denial =
        decode_package_archive(&bytes, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
    );
    assert_eq!(denial.compatibility().unwrap().observed_version(), 15);
}
