pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::native_aspect_contract_record::*;

#[test]
fn native_aspect_nested_work_is_symmetric_and_failed_attempts_are_atomic() {
    let exported = native_package().export_typed_records().unwrap();
    let native = native_view(&exported);
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(native, defaults).unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(defaults);
    decoder.decode_frame(&bytes).unwrap();
    let nested_entries = decoder.work().nested_entries();
    assert_eq!(nested_entries, 4);

    let exact = defaults.with_maximum_nested_entries(nested_entries);
    assert_eq!(encode_record_frame(native, exact).unwrap(), bytes);
    let mut exact_decoder = WorthQueryPackageArchiveRecordDecoder::new(exact);
    exact_decoder.decode_frame(&bytes).unwrap();
    assert_eq!(exact_decoder.work().nested_entries(), nested_entries);

    let narrow = defaults.with_maximum_nested_entries(nested_entries - 1);
    assert_eq!(
        encode_record_frame(native, narrow).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
    let mut narrow_decoder = WorthQueryPackageArchiveRecordDecoder::new(narrow);
    assert_eq!(
        narrow_decoder.decode_frame(&bytes).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
    assert_eq!(
        narrow_decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn noncanonical_retained_fields_and_unknown_binding_fail_closed() {
    let exported = native_package().export_typed_records().unwrap();
    let native = native_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(native, limits).unwrap();
    let field_set = [
        0, 0, 0, 2, 0, 0, 0, 3, b'A', b'g', b'e', 0, 0, 0, 3, b'T', b'a', b'g',
    ];
    let offset = bytes
        .windows(field_set.len())
        .rposition(|window| window == field_set)
        .unwrap();

    let mut reordered = bytes.clone();
    reordered[offset + 8..offset + 11].copy_from_slice(b"Tag");
    reordered[offset + 15..offset + 18].copy_from_slice(b"Age");
    assert_decode_kind(
        &reordered,
        WorthQueryPackageArchiveDenialKind::NonCanonicalRecordSequence,
    );

    let mut duplicate = bytes.clone();
    duplicate[offset + 15..offset + 18].copy_from_slice(b"Age");
    assert_decode_kind(
        &duplicate,
        WorthQueryPackageArchiveDenialKind::NonCanonicalRecordSequence,
    );

    let mut unknown_binding = bytes;
    let binding_offset = unknown_binding.len() - (2 + 4 + "Profile".len());
    unknown_binding[binding_offset..binding_offset + 2].copy_from_slice(&9_u16.to_be_bytes());
    assert_decode_kind(
        &unknown_binding,
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant,
    );
}

#[test]
fn native_aspect_semantic_tamper_decodes_but_cannot_mint_package_identity() {
    let source = native_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut frames = exported
        .views()
        .map(|view| encode_record_frame(view, limits).unwrap())
        .collect::<Vec<_>>();
    let native_index = exported
        .views()
        .position(|view| view.family() == Family::NativeAspectContract)
        .unwrap();
    let schema_offset = frames[native_index]
        .windows(NativeSchema::NAME.len())
        .position(|window| window == NativeSchema::NAME.as_bytes())
        .unwrap();
    frames[native_index][schema_offset] = b'X';

    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        decode_manifest_frame(
            &encode_manifest_frame(exported.manifest(), limits).unwrap(),
            limits,
        )
        .unwrap(),
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    for frame in frames {
        let (index, record) = decoder.decode_frame(&frame).unwrap().into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }
    assert!(reconstruction
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .is_err());
}
