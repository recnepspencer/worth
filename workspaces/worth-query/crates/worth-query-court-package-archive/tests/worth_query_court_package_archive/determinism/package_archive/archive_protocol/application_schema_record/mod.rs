pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::application_schema_record::*;

#[test]
fn application_schema_frame_is_deterministic_exact_and_freshly_readmitted() {
    let source = schema_package();
    let exported = source.export_typed_records().unwrap();
    let schema = schema_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let first = encode_record_frame(schema, limits).unwrap();
    let second = encode_record_frame(schema, limits).unwrap();
    assert_eq!(first, second);

    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    let decoded = decoder.decode_frame(&first).unwrap();
    assert_eq!(decoded.canonical_index(), schema.canonical_index());
    assert_eq!(decoded.record(), schema.record());

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
