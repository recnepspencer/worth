pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::artifact_contract_record::*;

#[test]
fn artifact_contract_frame_is_deterministic_exact_and_freshly_readmitted() {
    let retained16 = decode_hex(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-query-testkit-package-archive/src/package_archive/archive_protocol/artifact_contract_record/artifact_contract_v16.hex"))
        .trim(),
    );
    let denial =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT)
            .decode_frame(&retained16)
            .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
    );
    assert_eq!(denial.compatibility().unwrap().observed_version(), 16);

    let retained17 = decode_hex(
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../worth-query-testkit-package-archive/src/package_archive/archive_protocol/artifact_contract_record/artifact_contract_v17.hex"))
        .trim(),
    );
    let denial =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT)
            .decode_frame(&retained17)
            .unwrap_err();
    assert_eq!(
        denial.kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
    );
    assert_eq!(denial.compatibility().unwrap().observed_version(), 17);

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
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .unwrap();
    assert_eq!(reconstructed.identity(), source.identity());
}
