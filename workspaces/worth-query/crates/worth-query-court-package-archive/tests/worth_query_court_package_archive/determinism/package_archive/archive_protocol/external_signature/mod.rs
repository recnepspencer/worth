pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::external_signature::*;

#[test]
fn external_signature_reentry_reproduces_the_canonical_envelope() {
    let records = package_fixture::minimal_package()
        .export_typed_records()
        .unwrap();
    let unsigned = unsigned_envelope(&records, fixture_descriptor());
    let ordinary = unsigned
        .clone()
        .attach_signature(signature(0xa5), WorthQueryPackageEnvelopeLimits::DEFAULT)
        .unwrap();
    let expected =
        encode_package_release_envelope(&ordinary, WorthQueryPackageEnvelopeLimits::DEFAULT)
            .unwrap();

    let assembled = assemble_untrusted_package_release_envelope(
        unsigned.signing_payload(),
        signature(0xa5),
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    )
    .unwrap();
    let observed = encode_package_release_envelope(
        assembled.envelope(),
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    )
    .unwrap();

    assert_eq!(observed, expected);
    assert_eq!(assembled.signature(), &[0xa5; 64]);
    assert_eq!(
        assembled.expected_package_identity(),
        records.manifest().package_identity()
    );
}
