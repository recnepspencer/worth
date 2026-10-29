pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::external_signature::*;

#[test]
fn external_signature_reentry_rejects_noncanonical_or_unbounded_inputs() {
    let records = package_fixture::minimal_package()
        .export_typed_records()
        .unwrap();
    let unsigned = unsigned_envelope(&records, fixture_descriptor());
    let mut tampered = unsigned.signing_payload().to_vec();
    tampered[0] ^= 0xff;
    assert_eq!(
        assemble_untrusted_package_release_envelope(
            &tampered,
            signature(0xa5),
            WorthQueryPackageEnvelopeLimits::DEFAULT,
        )
        .unwrap_err()
        .kind(),
        WorthQueryPackageArchiveDenialKind::InvalidMagic
    );

    let narrow = WorthQueryPackageEnvelopeLimits::new(
        WorthQueryPackageEnvelopeLimits::DEFAULT.maximum_envelope_bytes(),
        WorthQueryPackageEnvelopeLimits::DEFAULT.maximum_archive_bytes(),
        WorthQueryPackageEnvelopeLimits::DEFAULT.maximum_descriptive_text_bytes(),
        WorthQueryPackageEnvelopeLimits::DEFAULT.maximum_requirements(),
        63,
    );
    assert_eq!(
        assemble_untrusted_package_release_envelope(
            unsigned.signing_payload(),
            signature(0xa5),
            narrow,
        )
        .unwrap_err()
        .kind(),
        WorthQueryPackageArchiveDenialKind::EnvelopeSignatureByteBudgetExceeded
    );
}
