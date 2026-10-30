pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::manifest_frame::*;

#[test]
fn current_manifest_frame_matches_the_frozen_golden_vector() {
    let bytes =
        encode_manifest_frame(&manifest(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    assert_eq!(encode_hex(&bytes), CURRENT_MANIFEST_FRAME_HEX);

    let independent_golden_bytes = decode_hex(CURRENT_MANIFEST_FRAME_HEX);
    let decoded = decode_manifest_frame(
        &independent_golden_bytes,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(
        decoded.version(),
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION
    );
    assert_eq!(decoded.package_identity().bytes(), &[0x41; 32]);
    assert_eq!(decoded.record_count(), 1);
    assert_eq!(decoded.canonical_source_bytes(), 0);
    assert_eq!(decoded.logical_export_bytes(), 64);
    for family in WorthQueryPortablePackageRecordFamily::ALL {
        let expected = u32::from(family == WorthQueryPortablePackageRecordFamily::DomainIdentity);
        assert_eq!(decoded.family_count(family), expected, "{family:?}");
    }
    WorthQueryPortablePackageReconstruction::begin(
        decoded,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
}

#[test]
fn malformed_or_downgraded_frames_fail_closed() {
    let bytes =
        encode_manifest_frame(&manifest(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    for length in 0..bytes.len() {
        assert!(
            decode_manifest_frame(&bytes[..length], WorthQueryPackageArchiveLimits::DEFAULT)
                .is_err()
        );
    }
    let mut wrong_magic = bytes.clone();
    wrong_magic[0] ^= 1;
    assert_eq!(
        decode_manifest_frame(&wrong_magic, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidMagic
    );
    let mut downgrade = bytes.clone();
    downgrade[9] = 0;
    assert_eq!(
        decode_manifest_frame(&downgrade, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedArchiveVersion
    );
    let mut manifest_downgrade = bytes.clone();
    manifest_downgrade[15] = 2;
    assert_eq!(
        decode_manifest_frame(&manifest_downgrade, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedManifestVersion
    );
    let mut wrong_length = bytes.clone();
    wrong_length[13] -= 1;
    assert_eq!(
        decode_manifest_frame(&wrong_length, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidManifestLength
    );
    let mut excessive_length = bytes.clone();
    excessive_length[13] += 1;
    assert_eq!(
        decode_manifest_frame(&excessive_length, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidManifestLength
    );
    let mut trailing = bytes;
    trailing.push(0);
    assert_eq!(
        decode_manifest_frame(&trailing, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::TrailingBytes
    );
}

#[test]
fn inconsistent_manifest_inventory_is_not_encoded() {
    let counts = [0; WorthQueryPortablePackageRecordFamily::ALL.len()];
    let inconsistent = WorthQueryPortablePackageManifest::from_untrusted_fields(
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
        WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes([0x41; 32]),
        1,
        0,
        64,
        counts,
    );
    assert_eq!(
        encode_manifest_frame(&inconsistent, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidFamilyCount
    );
}

#[test]
fn caller_work_ceilings_reject_claims_on_both_protocol_directions() {
    let mut counts = [0; WorthQueryPortablePackageRecordFamily::ALL.len()];
    counts[0] = 1;
    let claimed_work = WorthQueryPortablePackageManifest::from_untrusted_fields(
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
        WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes([0x41; 32]),
        1,
        32,
        64,
        counts,
    );
    let bytes =
        encode_manifest_frame(&claimed_work, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let logical_limit = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        63,
        defaults.maximum_canonical_work_bytes(),
    );
    let canonical_limit = WorthQueryPackageArchiveLimits::new(
        defaults.maximum_manifest_frame_bytes(),
        defaults.maximum_records(),
        defaults.maximum_logical_bytes(),
        31,
    );
    for (limits, expected) in [
        (
            logical_limit,
            WorthQueryPackageArchiveDenialKind::LogicalByteBudgetExceeded,
        ),
        (
            canonical_limit,
            WorthQueryPackageArchiveDenialKind::CanonicalWorkBudgetExceeded,
        ),
    ] {
        assert_eq!(
            encode_manifest_frame(&claimed_work, limits)
                .unwrap_err()
                .kind(),
            expected
        );
        assert_eq!(
            decode_manifest_frame(&bytes, limits).unwrap_err().kind(),
            expected
        );
    }
}

#[test]
fn caller_cannot_widen_frame_or_record_ceilings() {
    let beyond_default = WorthQueryPackageArchiveLimits::DEFAULT
        .maximum_records()
        .checked_add(1)
        .expect("the protocol record ceiling leaves space for this boundary probe");
    let mut counts = [0; WorthQueryPortablePackageRecordFamily::ALL.len()];
    counts[0] = beyond_default;
    let oversized = WorthQueryPortablePackageManifest::from_untrusted_fields(
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
        WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes([0x41; 32]),
        beyond_default,
        0,
        64,
        counts,
    );
    let attempted_widening =
        WorthQueryPackageArchiveLimits::new(u64::MAX, u32::MAX, u64::MAX, u64::MAX);
    let oversized_frame =
        vec![
            0;
            usize::try_from(WorthQueryPackageArchiveLimits::DEFAULT.maximum_manifest_frame_bytes(),)
                .unwrap() + 1
        ];
    assert_eq!(
        decode_manifest_frame(&oversized_frame, attempted_widening)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::ManifestFrameByteBudgetExceeded
    );
    assert_eq!(
        encode_manifest_frame(&oversized, attempted_widening)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::RecordBudgetExceeded
    );
}

#[test]
fn caller_cannot_widen_declared_work_ceilings() {
    let attempted_widening =
        WorthQueryPackageArchiveLimits::new(u64::MAX, u32::MAX, u64::MAX, u64::MAX);
    let logical_default = WorthQueryPackageArchiveLimits::DEFAULT.maximum_logical_bytes();
    let excessive_logical = WorthQueryPortablePackageManifest::from_untrusted_fields(
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
        WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes([0x41; 32]),
        1,
        0,
        logical_default + 1,
        {
            let mut counts = [0; WorthQueryPortablePackageRecordFamily::ALL.len()];
            counts[0] = 1;
            counts
        },
    );
    assert_eq!(
        encode_manifest_frame(&excessive_logical, attempted_widening)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::LogicalByteBudgetExceeded
    );

    let canonical_default = WorthQueryPackageArchiveLimits::DEFAULT.maximum_canonical_work_bytes();
    let excessive_canonical = WorthQueryPortablePackageManifest::from_untrusted_fields(
        WORTH_QUERY_PORTABLE_PACKAGE_MANIFEST_VERSION,
        WorthQueryPortableDomainPackageIdentity::from_untrusted_bytes([0x41; 32]),
        1,
        canonical_default + 1,
        0,
        {
            let mut counts = [0; WorthQueryPortablePackageRecordFamily::ALL.len()];
            counts[0] = 1;
            counts
        },
    );
    assert_eq!(
        encode_manifest_frame(&excessive_canonical, attempted_widening)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::CanonicalWorkBudgetExceeded
    );
}
