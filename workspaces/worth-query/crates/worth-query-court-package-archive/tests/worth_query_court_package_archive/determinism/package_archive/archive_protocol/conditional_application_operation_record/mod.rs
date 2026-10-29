pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::conditional_application_operation_record::*;

#[test]
fn conditional_application_operation_frame_is_deterministic_and_exact() {
    for (version, originals) in [
        (9_u16, VERSION_NINE_CONDITIONAL_BINDING_HEX),
        (6_u16, VERSION_SIX_CONDITIONAL_BINDING_HEX),
        (7_u16, VERSION_SEVEN_CONDITIONAL_BINDING_HEX),
        (13_u16, VERSION_THIRTEEN_CONDITIONAL_BINDING_HEX),
        (14_u16, VERSION_FOURTEEN_CONDITIONAL_BINDING_HEX),
        (16_u16, VERSION_SIXTEEN_CONDITIONAL_BINDING_HEX),
        (17_u16, VERSION_SEVENTEEN_CONDITIONAL_BINDING_HEX),
    ] {
        for original in originals.split('|') {
            let bytes = decode_hex(original);
            let denial =
                WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT)
                    .decode_frame(&bytes)
                    .unwrap_err();
            assert_eq!(
                denial.kind(),
                WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
            );
            assert_eq!(denial.compatibility().unwrap().observed_version(), version);
        }
    }
    let source = conditional_package();
    let exported = source.export_typed_records().unwrap();
    let view = conditional_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;

    let first = encode_record_frame(view, limits).unwrap();
    assert_eq!(encode_record_frame(view, limits).unwrap(), first);
    assert_eq!(encode_hex(&first), VERSION_EIGHTEEN_CONDITIONAL_BINDING_HEX);
    assert_eq!(u16::from_be_bytes(first[2..4].try_into().unwrap()), 9);

    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    let decoded = decoder
        .decode_frame(&decode_hex(VERSION_EIGHTEEN_CONDITIONAL_BINDING_HEX))
        .unwrap();
    assert_eq!(decoded.canonical_index(), view.canonical_index());
    assert_eq!(decoded.record(), view.record());
    assert_eq!(decoder.work().record_frames(), 1);
    assert_eq!(decoder.work().logical_bytes(), (first.len() - 12) as u64);
    assert_eq!(decoder.work().nested_entries(), 0);
}
