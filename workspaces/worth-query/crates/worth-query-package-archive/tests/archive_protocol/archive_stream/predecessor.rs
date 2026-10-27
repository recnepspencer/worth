//! Genuine historical production package, paired with the parent's current
//! all-family export/reconstruction/fresh-validation journey. See provenance.md.
use super::*;

#[test]
fn original_pre_actor_all_family_package_is_unsupported_intact_without_a_candidate() {
    let original = include_bytes!("preactor_2bc_all_families.wqpkg");
    let submitted = original.to_vec();
    let manifest = decode_manifest_frame(
        &original[..MANIFEST_FRAME_BYTES],
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    for family in Family::ALL {
        assert!(
            manifest.family_count(family) > 0,
            "historical export includes {family:?}"
        );
    }
    let denial = decode_package_archive(&submitted, WorthQueryPackageArchiveLimits::DEFAULT)
        .expect_err("unsupported frame cannot return an archive or reconstruction candidate");
    assert_eq!(
        denial.kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion
    );
    assert_eq!(denial.compatibility().unwrap().observed_version(), 6);
    assert_eq!(WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION, 18);
    assert_eq!(submitted, original.as_slice());
}
