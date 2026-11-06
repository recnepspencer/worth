pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::native_aspect_contract_record::*;

#[test]
fn malformed_native_aspect_payloads_fail_without_committing_work() {
    let exported = native_package().export_typed_records().unwrap();
    let native = native_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let native_key_len = Profile::reference().name().len();
    let bytes = encode_record_frame(native, limits).unwrap();
    for length in 0..bytes.len() {
        assert!(WorthQueryPackageArchiveRecordDecoder::new(limits)
            .decode_frame(&bytes[..length])
            .is_err());
    }

    // Fixed law tags precede the shape. Unknown laws never reconstruct defaults.
    let law_offset = 12
        + 4
        + <NativeSchema as ApplicationSchema>::NAME.len()
        + 4
        + "NativeEntity".len()
        + 4
        + native_key_len
        + 4
        + native_key_len
        + 8
        + 8;
    for offset in [law_offset, law_offset + 2, law_offset + 4, law_offset + 6] {
        let mut unsupported = bytes.clone();
        unsupported[offset..offset + 2].copy_from_slice(&u16::MAX.to_be_bytes());
        let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
        assert_eq!(
            decoder.decode_frame(&unsupported).unwrap_err().kind(),
            WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant,
            "unknown law at byte {offset}; frame={}",
            encode_hex(&unsupported)
        );
        assert_eq!(
            decoder.work(),
            WorthQueryPackageArchiveDecodeWork::default()
        );
    }

    let mut invalid_aspect = bytes.clone();
    let aspect_offset =
        12 + 4 + <NativeSchema as ApplicationSchema>::NAME.len() + 4 + "NativeEntity".len() + 4;
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
