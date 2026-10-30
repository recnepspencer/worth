pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::conditional_application_operation_record::*;

#[test]
fn malformed_conditional_binding_payloads_fail_closed_without_committing_work() {
    let exported = conditional_package().export_typed_records().unwrap();
    let view = conditional_view(&exported);
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(view, defaults).unwrap();
    for length in 0..bytes.len() {
        assert!(WorthQueryPackageArchiveRecordDecoder::new(defaults)
            .decode_frame(&bytes[..length])
            .is_err());
    }
    let mut invalid_utf8 = bytes.clone();
    let owner_offset = 12 + 4;
    invalid_utf8[owner_offset] = 0xff;
    assert_eq!(
        WorthQueryPackageArchiveRecordDecoder::new(defaults)
            .decode_frame(&invalid_utf8)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidUtf8
    );
    let mut trailing = bytes;
    let payload_length = u32::from_be_bytes(trailing[8..12].try_into().unwrap()) + 1;
    trailing[8..12].copy_from_slice(&payload_length.to_be_bytes());
    trailing.push(0);
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(defaults);
    assert_eq!(
        decoder.decode_frame(&trailing).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::TrailingBytes
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}
