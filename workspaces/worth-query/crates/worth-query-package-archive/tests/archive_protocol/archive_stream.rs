#[path = "archive_stream/fixture.rs"]
pub(super) mod fixture;

use std::ops::Range;

use worth_query_installation::facade::{
    WorthQueryExpectedPortablePackageIdentity, WorthQueryPortablePackageReconstruction,
    WorthQueryPortablePackageReconstructionLimits, WorthQueryPortablePackageRecordFamily as Family,
};
use worth_query_package_archive::facade::*;

const MANIFEST_FRAME_BYTES: usize = 118;
const RECORD_FRAME_HEADER_BYTES: usize = 12;
const VERSION_TWO_MINIMAL_ARCHIVE_HEX: &str = include_str!("archive_stream/archive_v2.hex");

#[test]
fn version_two_minimal_archive_is_deterministic_and_frozen() {
    let export = fixture::minimal_package().export_typed_records().unwrap();
    let limits = WorthQueryPackageArchiveLimits::DEFAULT;
    let first = encode_package_archive(&export, limits).unwrap();
    assert_eq!(encode_package_archive(&export, limits).unwrap(), first);
    assert_eq!(encode_hex(&first), VERSION_TWO_MINIMAL_ARCHIVE_HEX.trim());
    let frozen = decode_hex(VERSION_TWO_MINIMAL_ARCHIVE_HEX.trim());
    let decoded = decode_package_archive(&frozen, limits).unwrap();
    assert_eq!(decoded.manifest(), export.manifest());
    assert_eq!(decoded.frames().len(), 1);
    assert_eq!(decoded.frames()[0].record(), &export.records()[0]);
}

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

#[test]
fn physical_and_cumulative_nested_budgets_are_symmetric() {
    let export = fixture::all_family_package()
        .export_typed_records()
        .unwrap();
    let defaults = WorthQueryPackageArchiveLimits::DEFAULT;
    let bytes = encode_package_archive(&export, defaults).unwrap();
    let work = decode_package_archive(&bytes, defaults)
        .unwrap()
        .decode_work();

    let exact_bytes = defaults.with_maximum_archive_bytes(bytes.len() as u64);
    assert_eq!(encode_package_archive(&export, exact_bytes).unwrap(), bytes);
    assert!(decode_package_archive(&bytes, exact_bytes).is_ok());
    let narrow_bytes = defaults.with_maximum_archive_bytes(bytes.len() as u64 - 1);
    assert_kind(
        encode_package_archive(&export, narrow_bytes).unwrap_err(),
        WorthQueryPackageArchiveDenialKind::ArchiveByteBudgetExceeded,
    );
    assert_kind(
        decode_package_archive(&bytes, narrow_bytes).unwrap_err(),
        WorthQueryPackageArchiveDenialKind::ArchiveByteBudgetExceeded,
    );

    let exact_nested = defaults.with_maximum_nested_entries(work.nested_entries());
    assert_eq!(
        encode_package_archive(&export, exact_nested).unwrap(),
        bytes
    );
    assert!(decode_package_archive(&bytes, exact_nested).is_ok());
    let narrow_nested = defaults.with_maximum_nested_entries(work.nested_entries() - 1);
    assert_kind(
        encode_package_archive(&export, narrow_nested).unwrap_err(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded,
    );
    assert_kind(
        decode_package_archive(&bytes, narrow_nested).unwrap_err(),
        WorthQueryPackageArchiveDenialKind::NestedEntryBudgetExceeded,
    );
}

#[test]
fn aggregate_inventory_rejects_reorder_duplicate_and_wrong_family() {
    let export = fixture::all_family_package()
        .export_typed_records()
        .unwrap();
    let bytes = encode_package_archive(&export, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let ranges = frame_ranges(&bytes);

    let mut reordered = bytes[..MANIFEST_FRAME_BYTES].to_vec();
    reordered.extend_from_slice(&bytes[ranges[1].clone()]);
    reordered.extend_from_slice(&bytes[ranges[0].clone()]);
    reordered.extend_from_slice(&bytes[ranges[2].start..]);
    assert_decode_kind(
        &reordered,
        WorthQueryPackageArchiveDenialKind::NonCanonicalRecordSequence,
    );

    let mut duplicate_index = bytes.clone();
    duplicate_index[ranges[1].start + 4..ranges[1].start + 8].copy_from_slice(&0_u32.to_be_bytes());
    assert_decode_kind(
        &duplicate_index,
        WorthQueryPackageArchiveDenialKind::NonCanonicalRecordSequence,
    );

    let capability = ranges
        .iter()
        .find(|range| u16_at(&bytes, range.start + 2) == 2)
        .unwrap();
    let mut wrong_family = bytes;
    wrong_family[capability.start + 2..capability.start + 4].copy_from_slice(&3_u16.to_be_bytes());
    assert_decode_kind(
        &wrong_family,
        WorthQueryPackageArchiveDenialKind::RecordFamilyInventoryMismatch,
    );
}

#[test]
fn aggregate_versions_truncation_trailing_and_semantic_tamper_fail_closed() {
    let source = fixture::minimal_package();
    let export = source.export_typed_records().unwrap();
    let bytes = encode_package_archive(&export, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();

    assert!(decode_package_archive(
        &bytes[..bytes.len() - 1],
        WorthQueryPackageArchiveLimits::DEFAULT
    )
    .is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_decode_kind(&trailing, WorthQueryPackageArchiveDenialKind::TrailingBytes);
    let mut archive_version = bytes.clone();
    archive_version[8..10]
        .copy_from_slice(&(WORTH_QUERY_PACKAGE_ARCHIVE_PROTOCOL_VERSION + 1).to_be_bytes());
    assert_decode_kind(
        &archive_version,
        WorthQueryPackageArchiveDenialKind::UnsupportedArchiveVersion,
    );
    let mut record_version = bytes.clone();
    record_version[MANIFEST_FRAME_BYTES..MANIFEST_FRAME_BYTES + 2]
        .copy_from_slice(&(WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION - 1).to_be_bytes());
    assert_decode_kind(
        &record_version,
        WorthQueryPackageArchiveDenialKind::UnsupportedRecordVersion,
    );

    let mut tampered = bytes;
    let owner = b"archive.stream.minimal";
    let offset = tampered
        .windows(owner.len())
        .position(|window| window == owner)
        .unwrap();
    tampered[offset] = b'X';
    let decoded =
        decode_package_archive(&tampered, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
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

fn frame_ranges(bytes: &[u8]) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut start = MANIFEST_FRAME_BYTES;
    while start < bytes.len() {
        let payload = u32::from_be_bytes(bytes[start + 8..start + 12].try_into().unwrap()) as usize;
        let end = start + RECORD_FRAME_HEADER_BYTES + payload;
        ranges.push(start..end);
        start = end;
    }
    ranges
}

fn u16_at(bytes: &[u8], offset: usize) -> u16 {
    u16::from_be_bytes(bytes[offset..offset + 2].try_into().unwrap())
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::new(), |mut encoded, byte| {
        write!(&mut encoded, "{byte:02x}").unwrap();
        encoded
    })
}

fn decode_hex(encoded: &str) -> Vec<u8> {
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digits = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(digits, 16).unwrap()
        })
        .collect()
}

fn assert_decode_kind(bytes: &[u8], expected: WorthQueryPackageArchiveDenialKind) {
    assert_kind(
        decode_package_archive(bytes, WorthQueryPackageArchiveLimits::DEFAULT).unwrap_err(),
        expected,
    );
}

fn assert_kind(
    error: WorthQueryPackageArchiveDenial,
    expected: WorthQueryPackageArchiveDenialKind,
) {
    assert_eq!(error.kind(), expected);
}
