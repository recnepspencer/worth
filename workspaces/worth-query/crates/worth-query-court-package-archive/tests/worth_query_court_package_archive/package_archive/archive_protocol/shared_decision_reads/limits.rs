//! Production frame encode/decode retain depth, count and byte admission symmetrically.
use super::*;

#[test]
fn shared_definition_production_frames_enforce_depth_nested_count_and_length_limits() {
    let (source, _) = package(false);
    let exported = source.export_typed_records().unwrap();
    let view = || {
        exported
            .views()
            .find(|view| view.family() == WorthQueryPortablePackageRecordFamily::ApplicationSchema)
            .unwrap()
    };
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(view(), defaults).unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(defaults);
    assert_eq!(
        decoder.decode_frame(&bytes).unwrap().record(),
        view().record()
    );
    let nested = decoder.work().nested_entries();
    assert!(nested > 2);
    let exact = defaults
        .with_maximum_nested_entries(nested)
        .with_maximum_nesting_depth(2);
    assert!(encode_record_frame(view(), exact).is_ok());
    assert!(WorthQueryPackageArchiveRecordDecoder::new(exact)
        .decode_frame(&bytes)
        .is_ok());
    for (limits, kind) in [
        (
            defaults.with_maximum_nesting_depth(1),
            WorthQueryPackageArchiveDenialKind::NestingDepthBudgetExceeded,
        ),
        (
            defaults.with_maximum_nested_entries(nested - 1),
            WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded,
        ),
    ] {
        assert_eq!(
            encode_record_frame(view(), limits).unwrap_err().kind(),
            kind
        );
        assert_eq!(
            WorthQueryPackageArchiveRecordDecoder::new(limits)
                .decode_frame(&bytes)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    let length = bytes.len() as u64 - 12;
    let exact = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        length,
        defaults.maximum_canonical_work_bytes(),
    );
    assert!(encode_record_frame(view(), exact).is_ok());
    assert!(WorthQueryPackageArchiveRecordDecoder::new(exact)
        .decode_frame(&bytes)
        .is_ok());
    let short = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        length - 1,
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(
        encode_record_frame(view(), short).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::RecordFrameByteBudgetExceeded
    );
    assert_eq!(
        WorthQueryPackageArchiveRecordDecoder::new(short)
            .decode_frame(&bytes)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::RecordFrameByteBudgetExceeded
    );
}

#[test]
fn shared_definition_count_and_text_claims_refuse_truncation_in_production_decoder() {
    let (source, _) = package(false);
    let exported = source.export_typed_records().unwrap();
    let view = exported
        .views()
        .find(|view| view.family() == WorthQueryPortablePackageRecordFamily::ApplicationSchema)
        .unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(view, defaults).unwrap();
    let occurrences = bytes
        .windows(4)
        .enumerate()
        .filter_map(|(i, bytes)| (bytes == [0, 55, 0, 1]).then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(
        occurrences.len(),
        1,
        "this fixture has exactly one new definition header"
    );
    let start = occurrences[0];
    assert_eq!(&bytes[start + 4..start + 8], &2_u32.to_be_bytes());
    // Entity targets One/Two each occupy u16 tag + u32 length + three UTF-8 bytes.
    let derived = start + 8 + 2 * (2 + 4 + 3);
    assert_eq!(&bytes[derived..derived + 4], &0_u32.to_be_bytes());
    for offset in [start + 4, derived] {
        let mut poisoned = bytes.clone();
        poisoned[offset..offset + 4].copy_from_slice(&1_000_000_u32.to_be_bytes());
        assert_eq!(
            WorthQueryPackageArchiveRecordDecoder::new(defaults)
                .decode_frame(&poisoned)
                .unwrap_err()
                .kind(),
            WorthQueryPackageArchiveDenialKind::Truncated
        );
    }
    let mut text = bytes.clone();
    text[start + 10..start + 14].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(
        WorthQueryPackageArchiveRecordDecoder::new(defaults)
            .decode_frame(&text)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::Truncated
    );
    let mut version = bytes;
    version[start + 2..start + 4].copy_from_slice(&2_u16.to_be_bytes());
    version[start + 4..start + 8].copy_from_slice(&u32::MAX.to_be_bytes());
    assert_eq!(
        WorthQueryPackageArchiveRecordDecoder::new(defaults)
            .decode_frame(&version)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
    );
    version[start + 2..start + 4].copy_from_slice(&1_u16.to_be_bytes());
    assert_eq!(
        WorthQueryPackageArchiveRecordDecoder::new(defaults)
            .decode_frame(&version)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
}
