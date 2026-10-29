pub(crate) use worth_query_worlds::facade::package_archive::archive_protocol::native_aspect_contract_record::*;

#[test]
fn malformed_native_aspect_payloads_fail_without_committing_work() {
    let exported = native_package().export_typed_records().unwrap();
    let native = native_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(native, limits).unwrap();
    for length in 0..bytes.len() {
        assert!(WorthQueryPackageArchiveRecordDecoder::new(limits)
            .decode_frame(&bytes[..length])
            .is_err());
    }

    // Fixed law tags precede the shape. Unknown laws never reconstruct defaults.
    let law_offset = 12
        + 4
        + NativeSchema::NAME.len()
        + 4
        + "NativeEntity".len()
        + 4
        + "Profile".len()
        + 4
        + "Profile".len()
        + 8
        + 8;
    for offset in [law_offset, law_offset + 2, law_offset + 4, law_offset + 6] {
        let mut unsupported = bytes.clone();
        unsupported[offset..offset + 2].copy_from_slice(&u16::MAX.to_be_bytes());
        assert_decode_kind(
            &unsupported,
            WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant,
        );
    }

    let mut invalid_aspect = bytes.clone();
    let aspect_offset = 12 + 4 + NativeSchema::NAME.len() + 4 + "NativeEntity".len() + 4;
    invalid_aspect[aspect_offset] = b' ';
    assert_decode_kind(
        &invalid_aspect,
        WorthQueryPackageArchiveDenialKind::InvalidRecordShape,
    );

    let mut trailing = bytes;
    let payload_length = u32::from_be_bytes(trailing[8..12].try_into().unwrap()) + 1;
    trailing[8..12].copy_from_slice(&payload_length.to_be_bytes());
    trailing.push(0);
    assert_decode_kind(&trailing, WorthQueryPackageArchiveDenialKind::TrailingBytes);
}
