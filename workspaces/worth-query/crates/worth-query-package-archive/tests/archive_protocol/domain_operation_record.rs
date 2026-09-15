#[path = "domain_operation_record/fixture.rs"]
pub(super) mod fixture;

use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecord,
    WorthQueryPortablePackageRecordFamily as Family,
};
use worth_query_package_archive::facade::*;

use fixture::{collection_operation_package, operation_package};

#[test]
fn version_four_domain_operation_frame_matches_and_decodes_the_frozen_vector() {
    let source = operation_package();
    let exported = source.export_typed_records().unwrap();
    let operation = exported
        .views()
        .find(|view| view.family() == Family::DomainOperation)
        .unwrap();
    let bytes = encode_record_frame(operation, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let golden = decode_hex(include_str!("domain_operation_record/domain_operation_v4.hex").trim());
    assert_eq!(bytes, golden);
    let decoded =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT)
            .decode_frame(&golden)
            .unwrap();
    assert_eq!(decoded.record(), operation.record());
}

#[test]
fn domain_operation_frame_is_deterministic_exact_and_freshly_readmitted() {
    let source = operation_package();
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

    assert_eq!(
        decoder.work().record_frames(),
        u32::try_from(exported.records().len()).unwrap()
    );
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
fn domain_operation_payload_tamper_cannot_mint_query_identity() {
    let source = operation_package();
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
    let operation_index = exported
        .views()
        .position(|view| view.family() == Family::DomainOperation)
        .unwrap();
    let canonical_identity = match &exported.records()[operation_index] {
        WorthQueryPortablePackageRecord::DomainOperation(record) => record.canonical_identity(),
        _ => unreachable!(),
    };
    let operation_frame = &mut frames[operation_index];
    let identity_tail = operation_frame
        .windows(canonical_identity.len())
        .rposition(|window| window == canonical_identity.as_bytes())
        .expect("fixture carries its canonical operation identity");
    operation_frame[identity_tail] = b'W';

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
    assert!(reconstruction.close().unwrap().materialize().is_err());
}

#[test]
fn failed_domain_operation_decode_does_not_commit_cumulative_work() {
    let source = operation_package();
    let exported = source.export_typed_records().unwrap();
    let operation = exported
        .views()
        .find(|view| view.family() == Family::DomainOperation)
        .unwrap();
    let bytes = encode_record_frame(operation, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
    assert!(decoder.decode_frame(&bytes[..bytes.len() - 1]).is_err());
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
    assert!(decoder.decode_frame(&bytes).is_ok());
    assert_eq!(decoder.work().record_frames(), 1);
}

#[test]
fn nested_entry_budget_is_claimed_before_domain_operation_allocation() {
    let source = operation_package();
    let exported = source.export_typed_records().unwrap();
    let operation = exported
        .views()
        .find(|view| view.family() == Family::DomainOperation)
        .unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(operation, defaults).unwrap();
    let limits = defaults.with_maximum_nested_entries(0);
    assert_eq!(
        encode_record_frame(operation, limits).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
    let mut decoder = WorthQueryPackageArchiveRecordDecoder::new(limits);
    assert_eq!(
        decoder.decode_frame(&bytes).unwrap_err().kind(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded
    );
    assert_eq!(
        decoder.work(),
        WorthQueryPackageArchiveDecodeWork::default()
    );
}

#[test]
fn collection_field_paths_have_symmetric_encode_and_decode_work() {
    let source = collection_operation_package();
    let exported = source.export_typed_records().unwrap();
    let operation = exported
        .views()
        .find(|view| view.family() == Family::DomainOperation)
        .unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_record_frame(operation, defaults).unwrap();
    let mut permissive = WorthQueryPackageArchiveRecordDecoder::new(defaults);
    permissive.decode_frame(&bytes).unwrap();
    let nested_entries = permissive.work().nested_entries();
    assert!(nested_entries > 0);

    let exact = defaults.with_maximum_nested_entries(nested_entries);
    assert_eq!(encode_record_frame(operation, exact).unwrap(), bytes);
    let mut exact_decoder = WorthQueryPackageArchiveRecordDecoder::new(exact);
    exact_decoder.decode_frame(&bytes).unwrap();
    assert_eq!(exact_decoder.work().nested_entries(), nested_entries);

    let narrow = defaults.with_maximum_nested_entries(nested_entries - 1);
    assert_eq!(
        encode_record_frame(operation, narrow).unwrap_err().kind(),
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

fn decode_hex(encoded: &str) -> Vec<u8> {
    assert_eq!(encoded.len() % 2, 0);
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
