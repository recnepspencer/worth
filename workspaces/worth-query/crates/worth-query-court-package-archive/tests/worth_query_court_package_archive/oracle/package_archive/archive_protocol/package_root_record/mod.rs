pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::package_root_record::*;

#[test]
fn malformed_record_headers_and_payloads_fail_closed() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let capability = exported
        .views()
        .find(|view| view.family() == Family::CapabilityRequirement)
        .unwrap();
    let bytes =
        encode_package_root_record_frame(capability, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap();
    for length in 0..bytes.len() {
        assert!(decode_package_root_record_frame(
            &bytes[..length],
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .is_err());
    }

    assert_mutation_denial(&bytes, 1, 0, Kind::UnsupportedRecordVersion);
    assert_mutation_denial(&bytes, 1, 2, Kind::UnsupportedRecordVersion);
    assert_mutation_denial(&bytes, 3, 0xff, Kind::UnsupportedRecordFamily);
    assert_mutation_denial(&bytes, 3, 6, Kind::Truncated);
    assert_mutation_denial(&bytes, 11, bytes[11] - 1, Kind::InvalidRecordLength);
    assert_mutation_denial(&bytes, 11, bytes[11] + 1, Kind::InvalidRecordLength);
    assert_mutation_denial(&bytes, 15, bytes[15] + 1, Kind::Truncated);
    assert_mutation_denial(&bytes, 15, bytes[15] - 1, Kind::TrailingBytes);

    let mut invalid_utf8 = bytes.clone();
    invalid_utf8[16] = 0xff;
    assert_eq!(
        decode_package_root_record_frame(&invalid_utf8, WorthQueryPackageArchiveLimits::DEFAULT,)
            .unwrap_err()
            .kind(),
        Kind::InvalidUtf8
    );

    let mut trailing_payload = bytes;
    trailing_payload[11] += 1;
    trailing_payload.push(0);
    assert_eq!(
        decode_package_root_record_frame(
            &trailing_payload,
            WorthQueryPackageArchiveLimits::DEFAULT,
        )
        .unwrap_err()
        .kind(),
        Kind::TrailingBytes
    );
}
