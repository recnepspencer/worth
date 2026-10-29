pub(crate) use worth_query_worlds::facade::package_archive::archive_protocol::package_root_record::*;

#[test]
fn package_root_frames_are_deterministic_and_reenter_fresh_validation() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let manifest_bytes = encode_manifest_frame(exported.manifest(), limits).unwrap();
    let manifest = decode_manifest_frame(&manifest_bytes, limits).unwrap();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();

    for view in exported.views() {
        let first = encode_package_root_record_frame(view, limits).unwrap();
        let second = encode_package_root_record_frame(view, limits).unwrap();
        assert_eq!(first, second);
        let decoded = decode_package_root_record_frame(&first, limits).unwrap();
        assert_eq!(decoded.canonical_index(), view.canonical_index());
        assert_eq!(decoded.family(), view.family());
        assert_eq!(decoded.record(), view.record());
        let (canonical_index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(canonical_index, record).unwrap();
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
fn version_eighteen_package_root_frames_match_and_decode_the_frozen_vectors() {
    for (version, originals) in [
        (9_u16, VERSION_NINE_PACKAGE_ROOT_FRAMES_HEX),
        (6_u16, VERSION_SIX_PACKAGE_ROOT_FRAMES_HEX),
        (7_u16, VERSION_SEVEN_PACKAGE_ROOT_FRAMES_HEX),
        (13_u16, VERSION_THIRTEEN_PACKAGE_ROOT_FRAMES_HEX),
        (14_u16, VERSION_FOURTEEN_PACKAGE_ROOT_FRAMES_HEX),
        (16_u16, VERSION_SIXTEEN_PACKAGE_ROOT_FRAMES_HEX),
        (17_u16, VERSION_SEVENTEEN_PACKAGE_ROOT_FRAMES_HEX),
    ] {
        for original in originals.split('|') {
            let bytes = decode_hex(original);
            let denial =
                decode_package_root_record_frame(&bytes, WorthQueryPackageArchiveLimits::DEFAULT)
                    .unwrap_err();
            assert_eq!(
                denial.kind(),
                WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
            );
            assert_eq!(denial.compatibility().unwrap().observed_version(), version);
        }
    }
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let encoded = exported
        .views()
        .map(|view| {
            encode_hex(
                &encode_package_root_record_frame(view, WorthQueryPackageArchiveLimits::DEFAULT)
                    .unwrap(),
            )
        })
        .collect::<Vec<_>>()
        .join("|");
    assert_eq!(encoded, VERSION_EIGHTEEN_PACKAGE_ROOT_FRAMES_HEX);

    let independent_frames = VERSION_EIGHTEEN_PACKAGE_ROOT_FRAMES_HEX
        .split('|')
        .map(decode_hex)
        .collect::<Vec<_>>();
    assert_eq!(independent_frames.len(), exported.records().len());
    for (expected, bytes) in exported.views().zip(independent_frames) {
        let decoded =
            decode_package_root_record_frame(&bytes, WorthQueryPackageArchiveLimits::DEFAULT)
                .unwrap();
        assert_eq!(decoded.canonical_index(), expected.canonical_index());
        assert_eq!(decoded.family(), expected.family());
        assert_eq!(decoded.record(), expected.record());
    }
}

#[test]
fn unsupported_definition_kind_fails_before_phase_three() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let definition = exported
        .views()
        .find(|view| view.family() == Family::Definition)
        .unwrap();
    let mut bytes =
        encode_package_root_record_frame(definition, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap();
    for unsupported_tag in [4, 0xff] {
        bytes[13] = unsupported_tag;
        assert_eq!(
            decode_package_root_record_frame(&bytes, WorthQueryPackageArchiveLimits::DEFAULT)
                .unwrap_err()
                .kind(),
            Kind::UnsupportedDefinitionKind
        );
    }
}

#[test]
fn narrow_record_budget_rejects_before_payload_allocation() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let capability = exported
        .views()
        .find(|view| view.family() == Family::CapabilityRequirement)
        .unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_package_root_record_frame(capability, defaults).unwrap();
    let payload_length = u64::try_from(bytes.len() - 12).unwrap();
    let exact = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        payload_length,
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(
        encode_package_root_record_frame(capability, exact).unwrap(),
        bytes
    );
    assert!(decode_package_root_record_frame(&bytes, exact).is_ok());
    let narrow = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        payload_length - 1,
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(
        encode_package_root_record_frame(capability, narrow)
            .unwrap_err()
            .kind(),
        Kind::RecordFrameByteBudgetExceeded
    );
    assert_eq!(
        decode_package_root_record_frame(&bytes, narrow)
            .unwrap_err()
            .kind(),
        Kind::RecordFrameByteBudgetExceeded
    );
}

#[test]
fn canonical_index_ceiling_rejects_before_payload_decode() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let capability = exported
        .views()
        .find(|view| view.family() == Family::CapabilityRequirement)
        .unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_package_root_record_frame(capability, defaults).unwrap();
    let exact_index = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        capability.canonical_index() + 1,
        defaults.maximum_logical_bytes(),
        defaults.maximum_canonical_work_bytes(),
    );
    assert!(encode_package_root_record_frame(capability, exact_index).is_ok());
    assert!(decode_package_root_record_frame(&bytes, exact_index).is_ok());
    let no_records = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        0,
        defaults.maximum_logical_bytes(),
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(
        encode_package_root_record_frame(capability, no_records)
            .unwrap_err()
            .kind(),
        Kind::RecordIndexBudgetExceeded
    );
    assert_eq!(
        decode_package_root_record_frame(&bytes, no_records)
            .unwrap_err()
            .kind(),
        Kind::RecordIndexBudgetExceeded
    );
}

#[test]
fn phase_three_rejects_reordered_record_frames() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let frames = exported
        .views()
        .map(|view| encode_package_root_record_frame(view, limits).unwrap())
        .collect::<Vec<_>>();

    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    let second = decode_package_root_record_frame(&frames[1], limits).unwrap();
    let (index, record) = second.into_parts();
    assert!(matches!(
        reconstruction.push_record(index, record),
        Err(
            WorthQueryPortablePackageReconstructionDenial::RecordIndexMismatch {
                expected: 0,
                observed: 1,
            }
        )
    ));
}

#[test]
fn semantic_tamper_remains_untrusted_until_fresh_identity_validation() {
    let source = root_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut tampered = exported
        .views()
        .map(|view| encode_package_root_record_frame(view, limits).unwrap())
        .collect::<Vec<_>>();
    let capability_index = exported
        .views()
        .position(|view| view.family() == Family::CapabilityRequirement)
        .unwrap();
    tampered[capability_index][16] = b'x';
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for bytes in tampered {
        let decoded = decode_package_root_record_frame(&bytes, limits).unwrap();
        let (index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }
    assert!(matches!(
        reconstruction
            .close()
            .unwrap()
            .materialize()
            .unwrap()
            .validate_freshly(
                WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                    source.identity().clone(),
                ),
            ),
        Err(WorthQueryPortablePackageReconstructionDenial::ManifestPackageIdentityMismatch { .. })
    ));
}
