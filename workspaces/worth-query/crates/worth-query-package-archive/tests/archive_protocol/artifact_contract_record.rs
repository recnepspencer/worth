#[path = "artifact_contract_record/fixture.rs"]
pub(super) mod fixture;

use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecordFamily as Family,
};
use worth_query_package_archive::facade::*;

use fixture::artifact_package;

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
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .unwrap();
    assert_eq!(reconstructed.identity(), source.identity());
}

#[test]
fn version_three_artifact_contract_frame_matches_the_frozen_vector() {
    let source = artifact_package();
    let exported = source.export_typed_records().unwrap();
    let artifact = artifact_view(&exported);
    let bytes = encode_record_frame(artifact, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let golden = include_str!("artifact_contract_record/artifact_contract_v3.hex").trim();
    assert_eq!(encode_hex(&bytes), golden);
    assert_eq!(u16::from_be_bytes(bytes[0..2].try_into().unwrap()), 3);
    assert_eq!(u16::from_be_bytes(bytes[2..4].try_into().unwrap()), 7);
    assert_eq!(
        u32::from_be_bytes(bytes[4..8].try_into().unwrap()),
        artifact.canonical_index()
    );

    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert_eq!(
        decoder.decode_frame(&bytes).unwrap().record(),
        artifact.record()
    );
}

#[test]
fn artifact_tamper_remains_untrusted_until_expected_identity_comparison() {
    let source = artifact_package();
    let exported = source.export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let manifest = decode_manifest_frame(
        &encode_manifest_frame(exported.manifest(), limits).unwrap(),
        limits,
    )
    .unwrap();
    let mut frames = exported
        .views()
        .map(|view| encode_record_frame(view, limits).unwrap())
        .collect::<Vec<_>>();
    let artifact_index = exported
        .views()
        .position(|view| view.family() == Family::ArtifactContract)
        .unwrap();
    let family = b"worth.archive.candidates";
    let family_offset = frames[artifact_index]
        .windows(family.len())
        .position(|window| window == family)
        .unwrap();
    frames[artifact_index][family_offset] = b'W';

    let mut reconstruction = WorthQueryPortablePackageReconstruction::begin(
        manifest,
        WorthQueryPortablePackageReconstructionLimits::DEFAULT,
    )
    .unwrap();
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    for frame in frames {
        let decoded = decoder.decode_frame(&frame).unwrap();
        let (index, record) = decoded.into_parts();
        reconstruction = reconstruction.push_record(index, record).unwrap();
    }
    let candidate = reconstruction.close().unwrap().materialize().unwrap();
    assert!(candidate
        .validate_freshly(
            WorthQueryExpectedPortablePackageIdentity::from_untrusted_identity(
                source.identity().clone(),
            ),
        )
        .is_err());
}

#[test]
fn artifact_nested_work_is_symmetric_and_failed_attempts_do_not_commit() {
    let source = artifact_package();
    let exported = source.export_typed_records().unwrap();
    let artifact = artifact_view(&exported);
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(artifact, defaults).unwrap();
    let mut permissive = WorthQueryPackageArchiveRecordDecoder::new(defaults);
    permissive.decode_frame(&bytes).unwrap();
    let nested_entries = permissive.work().nested_entries();
    assert!(nested_entries > 0);

    let exact = defaults.with_maximum_nested_entries(nested_entries);
    assert_eq!(encode_record_frame(artifact, exact).unwrap(), bytes);
    let mut exact_decoder = WorthQueryPackageArchiveRecordDecoder::new(exact);
    exact_decoder.decode_frame(&bytes).unwrap();
    assert_eq!(exact_decoder.work().nested_entries(), nested_entries);

    let narrow = defaults.with_maximum_nested_entries(nested_entries - 1);
    assert_eq!(
        encode_record_frame(artifact, narrow).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
    let mut narrow_decoder = WorthQueryPackageArchiveRecordDecoder::new(narrow);
    assert_eq!(
        narrow_decoder.decode_frame(&bytes).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
    assert_eq!(
        narrow_decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn artifact_unknown_variant_and_trailing_payload_fail_closed() {
    let source = artifact_package();
    let exported = source.export_typed_records().unwrap();
    let artifact = artifact_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(artifact, limits).unwrap();

    let mut unknown = bytes.clone();
    let content_identity_offset = 12 + 4 + "worth.archive.candidates".len() + 4 + 4;
    unknown[content_identity_offset..content_identity_offset + 2]
        .copy_from_slice(&u16::MAX.to_be_bytes());
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    assert_eq!(
        decoder.decode_frame(&unknown).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );

    let mut trailing = bytes;
    let payload_length = u32::from_be_bytes(trailing[8..12].try_into().unwrap());
    trailing[8..12].copy_from_slice(&(payload_length + 1).to_be_bytes());
    trailing.push(0);
    assert_eq!(
        decoder.decode_frame(&trailing).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::TrailingBytes
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn artifact_decoder_rejects_noncanonical_sequences_without_normalizing() {
    let source = artifact_package();
    let exported = source.export_typed_records().unwrap();
    let artifact = artifact_view(&exported);
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let mut bytes = encode_record_frame(artifact, limits).unwrap();
    let canonical_substitutions = [0, 0, 0, 2, 0, 1, 0, 2];
    let offset = bytes
        .windows(canonical_substitutions.len())
        .position(|window| window == canonical_substitutions)
        .expect("fixture carries both canonical substitution postures");
    bytes[offset + 4..offset + 8].copy_from_slice(&[0, 2, 0, 1]);

    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    assert_eq!(
        decoder.decode_frame(&bytes).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NonCanonicalRecordSequence
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

fn artifact_view<'a>(
    exported: &'a worth_query_installation::facade::WorthQueryPortablePackageRecordSet,
) -> worth_query_installation::facade::WorthQueryPortablePackageRecordView<'a> {
    exported
        .views()
        .find(|view| view.family() == Family::ArtifactContract)
        .unwrap()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
