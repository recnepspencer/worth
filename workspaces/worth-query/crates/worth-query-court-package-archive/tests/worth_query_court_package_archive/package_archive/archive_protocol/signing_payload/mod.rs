pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::signing_payload::*;

#[test]
fn canonical_signing_payload_reenters_as_the_same_untrusted_description() {
    let records = fixture::all_family_package()
        .export_typed_records()
        .unwrap();
    let produced = unsigned_envelope(&records, fixture_descriptor());

    let decoded = decode_package_release_signing_payload(
        produced.signing_payload(),
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    )
    .unwrap();

    assert_eq!(decoded.unsigned(), &produced);
    assert_eq!(decoded.signing_payload(), produced.signing_payload());
    assert_eq!(
        decoded.expected_package_identity(),
        records.manifest().package_identity()
    );
}

#[test]
fn signing_payload_reentry_rejects_signature_suffix_downgrade_and_corruption() {
    let records = fixture::minimal_package().export_typed_records().unwrap();
    let produced = unsigned_envelope(&records, fixture_descriptor());
    let complete = signed_envelope_bytes(&records, fixture_descriptor());
    assert_kind(
        &complete,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
        WorthQueryPackageArchiveDenialKind::TrailingBytes,
    );

    let mut downgrade = produced.signing_payload().to_vec();
    downgrade[8..10].copy_from_slice(&2_u16.to_be_bytes());
    assert_kind(
        &downgrade,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
        WorthQueryPackageArchiveDenialKind::UnsupportedEnvelopeVersion,
    );

    let mut corrupt = produced.signing_payload().to_vec();
    let archive = corrupt
        .windows(8)
        .position(|window| window == b"WQPKGAR\0")
        .unwrap();
    corrupt[archive + 12] ^= 1;
    assert_kind(
        &corrupt,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
        WorthQueryPackageArchiveDenialKind::ArchiveChecksumMismatch,
    );
}

#[test]
fn signing_payload_reentry_enforces_its_exact_total_byte_ceiling() {
    let records = fixture::ordered_requirement_package()
        .export_typed_records()
        .unwrap();
    let produced = unsigned_envelope(&records, fixture_descriptor());
    let bytes = produced.signing_payload();
    let defaults = WorthQueryPackageEnvelopeLimits::DEFAULT;
    let exact = WorthQueryPackageEnvelopeLimits::new(
        bytes.len() as u64,
        defaults.maximum_archive_bytes(),
        defaults.maximum_descriptive_text_bytes(),
        defaults.maximum_requirements(),
        defaults.maximum_signature_bytes(),
    );
    assert!(decode_package_release_signing_payload(bytes, exact).is_ok());

    let narrow = WorthQueryPackageEnvelopeLimits::new(
        bytes.len() as u64 - 1,
        defaults.maximum_archive_bytes(),
        defaults.maximum_descriptive_text_bytes(),
        defaults.maximum_requirements(),
        defaults.maximum_signature_bytes(),
    );
    assert_kind(
        bytes,
        narrow,
        WorthQueryPackageArchiveDenialKind::EnvelopeByteBudgetExceeded,
    );
}

#[test]
fn unsigned_envelope_admits_the_expected_signature_shape_before_host_effects() {
    let records = fixture::minimal_package().export_typed_records().unwrap();
    let produced = unsigned_envelope(&records, fixture_descriptor());
    let defaults = WorthQueryPackageEnvelopeLimits::DEFAULT;
    assert_eq!(
        produced
            .require_external_signature_capacity(0, defaults)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::EmptyEnvelopeSignature
    );
    assert_eq!(
        produced
            .require_external_signature_capacity(defaults.maximum_signature_bytes() + 1, defaults)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::EnvelopeSignatureByteBudgetExceeded
    );

    let exact_complete_bytes = produced.signing_payload().len() as u64 + 4 + 64;
    let exact = WorthQueryPackageEnvelopeLimits::new(
        exact_complete_bytes,
        defaults.maximum_archive_bytes(),
        defaults.maximum_descriptive_text_bytes(),
        defaults.maximum_requirements(),
        defaults.maximum_signature_bytes(),
    );
    assert!(produced
        .require_external_signature_capacity(64, exact)
        .is_ok());
    let narrow = WorthQueryPackageEnvelopeLimits::new(
        exact_complete_bytes - 1,
        defaults.maximum_archive_bytes(),
        defaults.maximum_descriptive_text_bytes(),
        defaults.maximum_requirements(),
        defaults.maximum_signature_bytes(),
    );
    assert_eq!(
        produced
            .require_external_signature_capacity(64, narrow)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::EnvelopeByteBudgetExceeded
    );
}
