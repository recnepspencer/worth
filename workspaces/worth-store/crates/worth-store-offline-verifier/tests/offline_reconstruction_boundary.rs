use sha2::Digest;
use std::path::Path;

#[test]
fn physical_format_has_no_offline_walk_or_reconstruction_exports() {
    let format = Path::new(env!("CARGO_MANIFEST_DIR")).join("../worth-store-physical-format/src");
    for name in ["mod.rs", "bounded_decode.rs", "structural_observation.rs"] {
        assert!(
            !format.join("offline_walk").join(name).exists(),
            "retired {name}"
        );
    }
    let facade = std::fs::read_to_string(format.join("lib.rs")).unwrap();
    for retired in [
        "mod offline_walk",
        "pub use offline_walk",
        "verify_bounded_extent_artifact_from_reader",
        "verify_bounded_page_artifact_from_reader",
        "verify_bounded_root_manifest_artifact_from_reader",
        "OfflineStructuralObservation",
        "OfflinePhysicalArtifactFamily",
    ] {
        assert!(
            !facade.contains(retired),
            "format must not retain {retired}"
        );
    }
}

#[test]
fn structural_observation_is_owned_by_verifier_without_validation_authority() {
    use worth_store_offline_verifier::{
        classify_offline_artifact_family, observe_bounded_physical_bytes,
        OfflinePhysicalArtifactFamily,
    };
    let family = classify_offline_artifact_family("arbitrary.page");
    assert_eq!(family, OfflinePhysicalArtifactFamily::Page);
    let bytes = b"not a physical page";
    let observed = observe_bounded_physical_bytes(family, 17, bytes);
    assert_eq!(observed.offset(), 17);
    assert_eq!(observed.length(), bytes.len() as u64);
    assert_eq!(observed.family(), family);
    // The result is only a bounded content fingerprint, never a validator result.
    assert_eq!(
        observed.content_digest(),
        <[u8; 32]>::from(sha2::Sha256::digest(bytes))
    );
}
