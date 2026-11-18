pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::native_aspect_contract_record::*;

#[test]
fn native_aspect_frame_is_deterministic_exact_and_freshly_readmitted() {
    let source = native_package();
    let exported = source.export_typed_records().unwrap();
    let native = native_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let first = encode_record_frame(native, limits).unwrap();
    assert_eq!(encode_record_frame(native, limits).unwrap(), first);
    assert_eq!(encode_hex(&first), VERSION_NINETEEN_NATIVE_ASPECT_HEX);
    assert_eq!(u16::from_be_bytes(first[2..4].try_into().unwrap()), 11);
    let frozen = decode_hex(VERSION_NINETEEN_NATIVE_ASPECT_HEX);
    let decoded = WorthQueryPackageArchiveRecordDecoder::new(limits)
        .decode_frame(&frozen)
        .unwrap();
    assert_eq!(decoded.canonical_index(), native.canonical_index());
    assert_eq!(decoded.record(), native.record());

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
    for view in exported.views() {
        let decoded = decoder
            .decode_frame(&encode_record_frame(view, limits).unwrap())
            .unwrap();
        let (index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
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
