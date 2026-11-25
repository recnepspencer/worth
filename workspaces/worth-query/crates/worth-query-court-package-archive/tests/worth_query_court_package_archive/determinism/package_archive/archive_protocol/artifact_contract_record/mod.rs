pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::artifact_contract_record::*;

#[test]
fn artifact_contract_frame_is_deterministic_exact_and_freshly_readmitted() {
    let source = artifact_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();

    for view in exported.views() {
        let first = encode_record_frame(view, limits).unwrap();
        let second = encode_record_frame(view, limits).unwrap();
        assert_eq!(first, second);
        let decoded = decoder.decode_frame(&first).unwrap();
        assert_eq!(decoded.canonical_index(), view.canonical_index());
        assert_eq!(decoded.record(), view.record());
        let (index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }

    let reconstructed = reconstruction
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(WorthQueryExpectedPortablePackageIdentity::new(
            source.identity().clone(),
        ))
        .unwrap();
    assert_eq!(reconstructed.identity(), source.identity());
}
