pub(crate) use worth_query_worlds::facade::package_archive::archive_protocol::manifest_frame::*;

#[test]
fn manifest_frame_is_deterministic_and_reenters_phase_three() {
    let first =
        encode_manifest_frame(&manifest(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let second =
        encode_manifest_frame(&manifest(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    assert_eq!(first, second);
    let decoded = decode_manifest_frame(&first, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    assert_eq!(decoded.package_identity().bytes(), &[0x41; 32]);
    WorthQueryPortablePackageReconstruction::begin(
        decoded,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
}

#[test]
fn malformed_family_width_or_inventory_fails_closed() {
    const FAMILY_WIDTH_FINAL_BYTE: usize = 69;
    const RECORD_COUNT_FINAL_BYTE: usize = 51;

    let bytes =
        encode_manifest_frame(&manifest(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let mut wrong_family_width = bytes.clone();
    wrong_family_width[FAMILY_WIDTH_FINAL_BYTE] -= 1;
    assert_eq!(
        decode_manifest_frame(&wrong_family_width, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidFamilyCount
    );
    let mut inconsistent_inventory = bytes;
    inconsistent_inventory[RECORD_COUNT_FINAL_BYTE] += 1;
    assert_eq!(
        decode_manifest_frame(
            &inconsistent_inventory,
            WorthQueryPackageArchiveLimits::DEFAULT
        )
        .unwrap_err()
        .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidFamilyCount
    );
}

#[test]
fn caller_limits_reject_claims_before_phase_three() {
    let bytes =
        encode_manifest_frame(&manifest(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let limits = WorthQueryPackageArchiveLimits::new(4_096, 0, 64, 0);
    assert_eq!(
        decode_manifest_frame(&bytes, limits).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::RecordBudgetExceeded
    );
}

#[test]
fn caller_frame_ceiling_rejects_before_decode_or_encode() {
    let bytes =
        encode_manifest_frame(&manifest(), WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let below_frame = WorthQueryPackageArchiveLimits::new(
        u64::try_from(bytes.len()).unwrap() - 1,
        defaults.maximum_records(),
        defaults.maximum_logical_bytes(),
        defaults.maximum_canonical_work_bytes(),
    );
    assert_eq!(
        decode_manifest_frame(&bytes, below_frame)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::ManifestFrameByteBudgetExceeded
    );
    assert_eq!(
        encode_manifest_frame(&manifest(), below_frame)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::ManifestFrameByteBudgetExceeded
    );
}
