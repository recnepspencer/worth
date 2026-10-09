use sha2::{Digest, Sha256};
use worth_query_package_archive::facade::*;

use super::archive_stream::fixture;
use super::release_envelope_fixture::*;

const VERSION_ONE_RELEASE_ENVELOPE_HEX: &str =
    include_str!("release_envelope/release_envelope_v1.hex");
const ENVELOPE_HEADER_BYTES: usize = 18;
const EXPECTED_IDENTITY_BYTES: usize = 32;
const CHECKSUM_BYTES: usize = 32;

#[test]
fn version_one_release_envelope_is_deterministic_frozen_and_decodable() {
    // worth-query-installation/src/package/identity.rs:81 fixes identity v4;
    // encode the fixture archive and descriptor, then checksum the v1 envelope.
    let records = fixture::minimal_package().export_typed_records().unwrap();
    let first = signed_envelope_bytes(&records, fixture_descriptor());
    assert_eq!(signed_envelope_bytes(&records, fixture_descriptor()), first);
    assert_eq!(encode_hex(&first), VERSION_ONE_RELEASE_ENVELOPE_HEX.trim());

    let frozen = decode_hex(VERSION_ONE_RELEASE_ENVELOPE_HEX.trim());
    let decoded =
        decode_package_release_envelope(&frozen, WorthQueryPackageEnvelopeLimits::DEFAULT).unwrap();
    assert_eq!(decoded.signature(), &[0xa5; 64]);
    assert_eq!(
        decoded.expected_package_identity(),
        records.manifest().package_identity()
    );
    assert_eq!(
        decode_package_archive(decoded.archive(), WorthQueryPackageArchiveLimits::DEFAULT)
            .unwrap()
            .manifest(),
        records.manifest()
    );
}

#[test]
fn signing_payload_covers_descriptors_but_excludes_only_signature_bytes() {
    let records = fixture::minimal_package().export_typed_records().unwrap();
    let baseline = unsigned_envelope(&records, fixture_descriptor());
    let changed_build = unsigned_envelope(
        &records,
        descriptor_with_build(
            "rustc",
            "1.99.1",
            "stable",
            "1.99.1",
            "x86_64-pc-windows-msvc",
        ),
    );
    let changed_release = unsigned_envelope(
        &records,
        descriptor_with_release("workflow-editor", "2026.08.27"),
    );
    let changed_provenance = unsigned_envelope(
        &records,
        descriptor_with_provenance(
            "https://github.com/worth/core",
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "refs/tags/query-9.16.2",
        ),
    );
    let changed_signer = unsigned_envelope(&records, descriptor_with_signer("release-key-02"));
    for changed in [
        changed_build,
        changed_release,
        changed_provenance,
        changed_signer,
    ] {
        assert_ne!(changed.signing_payload(), baseline.signing_payload());
    }

    let first = baseline
        .clone()
        .attach_signature(signature(0xa5), WorthQueryPackageEnvelopeLimits::DEFAULT)
        .unwrap();
    let second = baseline
        .attach_signature(signature(0x5a), WorthQueryPackageEnvelopeLimits::DEFAULT)
        .unwrap();
    assert_eq!(first.signing_payload(), second.signing_payload());
    assert_ne!(
        encode_package_release_envelope(&first, WorthQueryPackageEnvelopeLimits::DEFAULT).unwrap(),
        encode_package_release_envelope(&second, WorthQueryPackageEnvelopeLimits::DEFAULT).unwrap()
    );
}

#[test]
fn release_requirements_are_derived_from_complete_typed_package_records() {
    let records = fixture::all_family_package()
        .export_typed_records()
        .unwrap();
    let unsigned = unsigned_envelope(&records, fixture_descriptor());
    let requirements = unsigned.requirements();
    assert_eq!(
        requirements
            .capabilities()
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["query-read"]
    );
    assert_eq!(
        requirements
            .configuration()
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["query"]
    );
    assert_eq!(
        requirements
            .operating()
            .iter()
            .map(|value| value.as_str())
            .collect::<Vec<_>>(),
        ["bounded"]
    );
    assert_eq!(
        requirements
            .execution_providers()
            .iter()
            .map(|value| (
                value.provider().as_str(),
                value.access_product().as_str(),
                value.allocator().as_str(),
            ))
            .collect::<Vec<_>>(),
        [("fixture-provider", "fixture-access", "fixture-arena")]
    );

    let signed = unsigned
        .attach_signature(signature(0xa5), WorthQueryPackageEnvelopeLimits::DEFAULT)
        .unwrap();
    let bytes =
        encode_package_release_envelope(&signed, WorthQueryPackageEnvelopeLimits::DEFAULT).unwrap();
    let decoded =
        decode_package_release_envelope(&bytes, WorthQueryPackageEnvelopeLimits::DEFAULT).unwrap();
    assert_eq!(
        decoded.envelope().unsigned().requirements(),
        signed.unsigned().requirements()
    );
}

#[test]
fn release_requirement_budget_denies_before_envelope_construction() {
    let records = fixture::all_family_package()
        .export_typed_records()
        .unwrap();
    let defaults = WorthQueryPackageEnvelopeLimits::DEFAULT;
    let no_requirements = WorthQueryPackageEnvelopeLimits::new(
        defaults.maximum_envelope_bytes(),
        defaults.maximum_archive_bytes(),
        defaults.maximum_descriptive_text_bytes(),
        0,
        defaults.maximum_signature_bytes(),
    );
    assert_eq!(
        prepare_package_release_envelope(
            &records,
            fixture_descriptor(),
            WorthQueryPackageArchiveLimits::DEFAULT,
            no_requirements,
        )
        .unwrap_err()
        .kind(),
        WorthQueryPackageArchiveDenialKind::EnvelopeRequirementBudgetExceeded
    );
}

#[test]
fn envelope_structure_rejects_downgrade_truncation_and_trailing_bytes() {
    let bytes = ordered_envelope_bytes();
    let mut downgrade = bytes.clone();
    downgrade[8..10].copy_from_slice(&2_u16.to_be_bytes());
    assert_decode_kind(
        &downgrade,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
        WorthQueryPackageArchiveDenialKind::UnsupportedEnvelopeVersion,
    );
    assert!(decode_package_release_envelope(
        &bytes[..bytes.len() - 1],
        WorthQueryPackageEnvelopeLimits::DEFAULT
    )
    .is_err());
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_decode_kind(
        &trailing,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
        WorthQueryPackageArchiveDenialKind::TrailingBytes,
    );
}

#[test]
fn envelope_requirement_sequences_reject_reorder_and_impossible_counts() {
    let bytes = ordered_envelope_bytes();
    let mut noncanonical = bytes.clone();
    let alpha = find_text(&noncanonical, b"alpha");
    let omega = find_text(&noncanonical, b"omega");
    noncanonical[alpha..alpha + 5].copy_from_slice(b"omega");
    noncanonical[omega..omega + 5].copy_from_slice(b"alpha");
    assert_decode_kind(
        &noncanonical,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
        WorthQueryPackageArchiveDenialKind::NonCanonicalEnvelopeRequirementSequence,
    );
    let mut impossible_count = bytes.clone();
    let alpha = find_text(&impossible_count, b"alpha");
    impossible_count[alpha - 8..alpha - 4].copy_from_slice(
        &WorthQueryPackageEnvelopeLimits::DEFAULT
            .maximum_requirements()
            .to_be_bytes(),
    );
    assert_decode_kind(
        &impossible_count,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
        WorthQueryPackageArchiveDenialKind::Truncated,
    );
}

#[test]
fn envelope_total_byte_limit_is_exact_and_symmetric() {
    let bytes = ordered_envelope_bytes();
    let defaults = WorthQueryPackageEnvelopeLimits::DEFAULT;
    let exact = WorthQueryPackageEnvelopeLimits::new(
        bytes.len() as u64,
        defaults.maximum_archive_bytes(),
        defaults.maximum_descriptive_text_bytes(),
        defaults.maximum_requirements(),
        defaults.maximum_signature_bytes(),
    );
    assert!(decode_package_release_envelope(&bytes, exact).is_ok());
    let signed = decode_package_release_envelope(&bytes, WorthQueryPackageEnvelopeLimits::DEFAULT)
        .unwrap()
        .into_envelope();
    assert_eq!(
        encode_package_release_envelope(&signed, exact).unwrap(),
        bytes
    );
    let narrow = WorthQueryPackageEnvelopeLimits::new(
        bytes.len() as u64 - 1,
        defaults.maximum_archive_bytes(),
        defaults.maximum_descriptive_text_bytes(),
        defaults.maximum_requirements(),
        defaults.maximum_signature_bytes(),
    );
    assert_decode_kind(
        &bytes,
        narrow,
        WorthQueryPackageArchiveDenialKind::EnvelopeByteBudgetExceeded,
    );
    assert_eq!(
        encode_package_release_envelope(&signed, narrow)
            .unwrap_err()
            .kind(),
        WorthQueryPackageArchiveDenialKind::EnvelopeByteBudgetExceeded
    );
}

#[test]
fn envelope_signature_limit_rejects_before_signature_allocation_on_decode() {
    let bytes = ordered_envelope_bytes();
    let defaults = WorthQueryPackageEnvelopeLimits::DEFAULT;
    let narrow_signature = WorthQueryPackageEnvelopeLimits::new(
        defaults.maximum_envelope_bytes(),
        defaults.maximum_archive_bytes(),
        defaults.maximum_descriptive_text_bytes(),
        defaults.maximum_requirements(),
        63,
    );
    assert_decode_kind(
        &bytes,
        narrow_signature,
        WorthQueryPackageArchiveDenialKind::EnvelopeSignatureByteBudgetExceeded,
    );
}

#[test]
fn envelope_preparation_preserves_the_tighter_archive_limit() {
    let records = fixture::ordered_requirement_package()
        .export_typed_records()
        .unwrap();
    let archive =
        encode_package_archive(&records, WorthQueryPackageArchiveLimits::DEFAULT).unwrap();
    let narrow_archive = WorthQueryPackageArchiveLimits::DEFAULT
        .with_maximum_archive_bytes(archive.len() as u64 - 1);
    assert_eq!(
        prepare_package_release_envelope(
            &records,
            fixture_descriptor(),
            narrow_archive,
            WorthQueryPackageEnvelopeLimits::DEFAULT,
        )
        .unwrap_err()
        .kind(),
        WorthQueryPackageArchiveDenialKind::ArchiveByteBudgetExceeded
    );
}

#[test]
fn archive_checksum_detects_corruption_but_cannot_create_trust() {
    let records = fixture::minimal_package().export_typed_records().unwrap();
    let bytes = signed_envelope_bytes(&records, fixture_descriptor());
    let archive_offset = find_text(&bytes, b"WQPKGAR\0");

    let mut corrupt = bytes.clone();
    corrupt[archive_offset + 12] ^= 1;
    assert_decode_kind(
        &corrupt,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
        WorthQueryPackageArchiveDenialKind::ArchiveChecksumMismatch,
    );

    let mut attacker_rewritten = corrupt;
    let archive_length = archive_length(&attacker_rewritten, archive_offset);
    let checksum: [u8; 32] =
        Sha256::digest(&attacker_rewritten[archive_offset..archive_offset + archive_length]).into();
    let checksum_offset = ENVELOPE_HEADER_BYTES + EXPECTED_IDENTITY_BYTES;
    attacker_rewritten[checksum_offset..checksum_offset + CHECKSUM_BYTES]
        .copy_from_slice(&checksum);
    let decoded = decode_package_release_envelope(
        &attacker_rewritten,
        WorthQueryPackageEnvelopeLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(decoded.signature(), &[0xa5; 64]);
    assert!(
        decode_package_archive(decoded.archive(), WorthQueryPackageArchiveLimits::DEFAULT).is_err()
    );
}

fn archive_length(bytes: &[u8], archive_offset: usize) -> usize {
    let length_offset = archive_offset - 8;
    usize::try_from(u64::from_be_bytes(
        bytes[length_offset..archive_offset].try_into().unwrap(),
    ))
    .unwrap()
}

fn find_text(bytes: &[u8], needle: &[u8]) -> usize {
    bytes
        .windows(needle.len())
        .position(|window| window == needle)
        .unwrap()
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

fn assert_decode_kind(
    bytes: &[u8],
    limits: WorthQueryPackageEnvelopeLimits,
    expected: WorthQueryPackageArchiveDenialKind,
) {
    assert_eq!(
        decode_package_release_envelope(bytes, limits)
            .unwrap_err()
            .kind(),
        expected
    );
}
