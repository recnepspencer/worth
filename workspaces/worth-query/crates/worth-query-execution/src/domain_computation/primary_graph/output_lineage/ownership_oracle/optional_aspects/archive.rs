use super::*;
use worth_foundational::facade::AbsenceLaw;
use worth_query_installation::facade::*;
use worth_query_package_archive::facade::*;
pub(super) fn roundtrip<Schema: BirthSchema>() {
    let source = WorthQueryPortableDomainPackage::new(WorthQueryPortableDomainIdentity::new(
        Schema::OWNER,
        Schema::MAJOR,
        Schema::MINOR,
    ))
    .application_schema(Schema::declaration().unwrap())
    .validate()
    .unwrap();
    let exported = source.export_typed_records().unwrap();
    let bytes = encode_package_archive(&exported, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let (manifest, frames) =
        decode_package_archive(&bytes, WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap()
            .into_parts();
    let mut intake = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for frame in frames {
        let (index, record) = frame.into_parts();
        intake = intake.push_record(index, record).unwrap();
    }
    let fresh = intake
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
    assert_eq!(fresh.identity(), source.identity());
    assert_eq!(fresh.export_typed_records().unwrap(), exported);
    let admitted =
        WorthQueryInstallationAdmissionProfile::new("optional-archive-support", "configuration")
            .admit(fresh)
            .unwrap();
    let installed = WorthQueryInstalledPackageIndex::build(
        WorthQueryInstallationRuntimeIdentity::fresh(),
        WorthQueryInstallationGeneration::initial(),
        [admitted],
    )
    .unwrap()
    .bind_application_schema(Schema::declaration().unwrap())
    .unwrap();
    let meanings = installed.value_meaning().unwrap();
    let node = meanings
        .iter()
        .find(|meaning| meaning.entity() == "OutputNode")
        .unwrap();
    assert_eq!(node.absence(), AbsenceLaw::Optional);
    assert_eq!(
        installed
            .native_contracts()
            .aspect("OutputNode", "NodeFacts")
            .unwrap()
            .contract()
            .absence(),
        AbsenceLaw::Optional
    );
    // A protocol7 record cannot be decoded under the current-only record reader.
    let mut frame = encode_record_frame(
        exported.views().next().unwrap(),
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    frame[..2].copy_from_slice(&7_u16.to_be_bytes());
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert!(decoder.decode_frame(&frame).is_err());
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}
