pub(crate) use worth_query_worlds::facade::package_archive::archive_protocol::conditional_application_operation_record::*;

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

#[test]
fn decoded_conditional_binding_reenters_only_through_fresh_package_validation() {
    let source = conditional_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let conditional = conditional_view(&exported);
    let decoded = WorthQueryPackageArchiveRecordDecoder::new(limits)
        .decode_frame(&encode_record_frame(conditional, limits).unwrap())
        .unwrap();
    let decoded_record = decoded.into_record();
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for view in exported.views() {
        let record = if view.family() == Family::ConditionalApplicationOperation {
            decoded_record.clone()
        } else {
            view.record().clone()
        };
        reconstruction = reconstruction
            .push_record(view.canonical_index(), record)
            .unwrap();
    }
    let reconstructed = reconstruction
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .unwrap();
    assert_eq!(reconstructed.identity(), source.identity());
}

#[test]
fn conditional_binding_byte_limit_is_symmetric_and_failures_do_not_commit() {
    let exported = conditional_package().export_typed_records().unwrap();
    let view = conditional_view(&exported);
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(view, defaults).unwrap();
    let payload_bytes = (bytes.len() - 12) as u64;
    let exact = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        payload_bytes,
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(encode_record_frame(view, exact).unwrap(), bytes);
    assert!(WorthQueryPackageArchiveRecordDecoder::new(exact)
        .decode_frame(&bytes)
        .is_ok());

    let narrow = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        payload_bytes - 1,
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(
        encode_record_frame(view, narrow).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::RecordFrameByteBudgetExceeded
    );
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(narrow);
    assert_eq!(
        decoder.decode_frame(&bytes).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::RecordFrameByteBudgetExceeded
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn semantic_tamper_decodes_but_cannot_mint_the_expected_package_identity() {
    let source = conditional_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let view = conditional_view(&exported);
    let mut bytes = encode_record_frame(view, limits).unwrap();
    let operation = Operation::IDENTIFIER.as_bytes();
    let offset = bytes
        .windows(operation.len())
        .position(|window| window == operation)
        .unwrap();
    bytes[offset] = b'X';
    let tampered = WorthQueryPackageArchiveRecordDecoder::new(limits)
        .decode_frame(&bytes)
        .unwrap()
        .into_record();
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for view in exported.views() {
        let record = if view.family() == Family::ConditionalApplicationOperation {
            tampered.clone()
        } else {
            view.record().clone()
        };
        reconstruction = reconstruction
            .push_record(view.canonical_index(), record)
            .unwrap();
    }
    let candidate = reconstruction.close().unwrap().materialize().unwrap();
    assert!(candidate
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .is_err());
}
