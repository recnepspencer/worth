pub(crate) use worth_query_testkit_package_archive::facade::package_archive::archive_protocol::application_operation_contract_record::*;

#[test]
fn production_export_operation_contract_frames_reenter_fresh_validation() {
    let source = super::conditional_application_operation_record::conditional_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
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

#[test]
fn operation_contract_semantic_tamper_decodes_but_cannot_mint_package_identity() {
    let source = super::conditional_application_operation_record::conditional_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut frames = exported
        .views()
        .map(|view| encode_record_frame(view, limits).unwrap())
        .collect::<Vec<_>>();
    let operation_index = exported
        .views()
        .position(|view| view.family() == Family::ApplicationOperationContract)
        .unwrap();
    let operation_name = match exported.records()[operation_index].clone() {
        WorthQueryPortablePackageRecord::ApplicationOperationContract(record) => {
            record.operation().to_owned()
        }
        _ => unreachable!(),
    };
    let offset = frames[operation_index]
        .windows(operation_name.len())
        .position(|window| window == operation_name.as_bytes())
        .unwrap();
    frames[operation_index][offset] = b'X';

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
    for frame in frames {
        let (index, record) = decoder.decode_frame(&frame).unwrap().into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }
    assert!(reconstruction
        .close()
        .unwrap()
        .materialize()
        .unwrap()
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .is_err());
}
