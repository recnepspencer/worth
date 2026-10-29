pub(crate) use worth_query_worlds::facade::package_archive::archive_protocol::archive_stream::*;

pub(crate) mod predecessor;

#[test]
fn all_record_families_cross_one_archive_and_reenter_fresh_query_validation() {
    let source = fixture::all_family_package();
    let export = source.export_typed_records().unwrap();
    for family in Family::ALL {
        assert!(export.manifest().family_count(family) > 0, "{family:?}");
    }
    let archive = encode_package_archive(&export, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let decoded =
        decode_package_archive(&archive, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    assert_eq!(
        decoded.decode_work().record_frames(),
        export.manifest().record_count()
    );
    let (manifest, frames) = decoded.into_parts();
    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    for frame in frames {
        let (index, record) = frame.into_parts();
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
