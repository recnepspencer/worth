use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejection, PhysicalIntegrityRejectionClass,
    PhysicalIntegrityVersionAxis,
};

use super::support::{
    assert_damage, assert_independent_frame_checksums, assert_rejected_counters, literal,
    rejection, scope, store, CLEAN_HEX, FOOTER_CHECKSUM_HEX, FRAME_OFFSET, GENERATION_HEX,
    HEADER_LENGTH_HEX, LENGTH_HEX, LSN_HEX, MAGIC_HEX, PAYLOAD_BYTE_HEX, PAYLOAD_CHECKSUM_HEX,
    SEGMENT_HEX, VERSION_HEX, ZERO_GENERATION_HEX, ZERO_SEGMENT_HEX,
};

#[test]
fn magic_and_family_denials_preserve_exact_scope() {
    let bytes = literal(MAGIC_HEX);
    let wal_scope = scope(7, 1, 2, bytes.len() as u64);
    let (magic_rejection, counters) = rejection(&bytes, wal_scope);
    assert_damage(
        magic_rejection,
        wal_scope,
        PhysicalDamageCause::WrongMagic,
        0,
        8,
        Some(PhysicalFormatField::Magic),
        PhysicalBlastRadius::CompleteArtifact,
    );
    assert_rejected_counters(counters, 151, PhysicalDamageCause::WrongMagic);

    let clean = literal(CLEAN_HEX);
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let selector_scope = PhysicalArtifactScope::current_root_selector(
        store(7),
        format,
        PhysicalByteRange::new(FRAME_OFFSET, clean.len() as u64).unwrap(),
    );
    let (rejection, _) = rejection(&clean, selector_scope);
    assert_damage(
        rejection,
        selector_scope,
        PhysicalDamageCause::FamilyMismatch,
        0,
        clean.len() as u64,
        None,
        PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn unsupported_version_is_not_collapsed_into_damage() {
    let bytes = literal(VERSION_HEX);
    assert_independent_frame_checksums(&bytes);
    let scope = scope(7, 1, 2, bytes.len() as u64);
    let (rejection, counters) = rejection(&bytes, scope);
    let PhysicalIntegrityRejection::Unsupported(unsupported) = rejection else {
        panic!("unsupported WAL version must retain its posture");
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), PhysicalIntegrityVersionAxis::WalFrame);
    assert_eq!(unsupported.observed(), 2);
    assert_eq!(counters.family(), PhysicalIntegrityArtifactFamily::WalFrame);
    assert_eq!(
        counters.rejected_for(PhysicalIntegrityRejectionClass::Unsupported),
        1
    );
}

#[test]
fn checksum_valid_framing_lies_name_the_encoded_length_field() {
    for (vector, offset) in [(HEADER_LENGTH_HEX, 10), (LENGTH_HEX, 44)] {
        let bytes = literal(vector);
        assert_independent_frame_checksums(&bytes);
        let scope = scope(7, 1, 2, bytes.len() as u64);
        let (rejection, counters) = rejection(&bytes, scope);
        assert_damage(
            rejection,
            scope,
            PhysicalDamageCause::FramingLengthMismatch,
            offset,
            if offset == 10 { 2 } else { 8 },
            Some(PhysicalFormatField::EncodedLength),
            PhysicalBlastRadius::CanonicalFrame,
        );
        assert_rejected_counters(counters, 151, PhysicalDamageCause::FramingLengthMismatch);
    }
}

#[test]
fn checksum_valid_segment_and_generation_substitution_are_distinct() {
    let cases = [
        (
            SEGMENT_HEX,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            12,
            PhysicalFormatField::SegmentIdentity,
        ),
        (
            GENERATION_HEX,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            20,
            PhysicalFormatField::PhysicalGeneration,
        ),
    ];
    for (vector, cause, offset, field) in cases {
        let bytes = literal(vector);
        assert_independent_frame_checksums(&bytes);
        let scope = scope(7, 1, 2, bytes.len() as u64);
        let (rejection, counters) = rejection(&bytes, scope);
        assert_damage(
            rejection,
            scope,
            cause,
            offset,
            8,
            Some(field),
            PhysicalBlastRadius::CompleteArtifact,
        );
        assert_rejected_counters(counters, 151, cause);
    }
}

#[test]
fn checksum_valid_zero_segment_and_generation_localize_separately() {
    let cases = [
        (
            ZERO_SEGMENT_HEX,
            PhysicalDamageCause::ArtifactIdentityMismatch,
            12,
            PhysicalFormatField::SegmentIdentity,
        ),
        (
            ZERO_GENERATION_HEX,
            PhysicalDamageCause::PhysicalGenerationMismatch,
            20,
            PhysicalFormatField::PhysicalGeneration,
        ),
    ];
    for (vector, cause, offset, field) in cases {
        let bytes = literal(vector);
        assert_independent_frame_checksums(&bytes);
        let scope = scope(7, 1, 2, bytes.len() as u64);
        let (rejection, counters) = rejection(&bytes, scope);
        assert_damage(
            rejection,
            scope,
            cause,
            offset,
            8,
            Some(field),
            PhysicalBlastRadius::CompleteArtifact,
        );
        assert_rejected_counters(counters, 151, cause);
    }
}

#[test]
fn checksum_valid_invalid_lsn_order_is_exposed_only_as_sequence_damage() {
    let bytes = literal(LSN_HEX);
    assert_independent_frame_checksums(&bytes);
    let scope = scope(7, 1, 2, bytes.len() as u64);
    let (rejection, counters) = rejection(&bytes, scope);
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::SequenceMismatch,
        28,
        16,
        Some(PhysicalFormatField::WalLsnRange),
        PhysicalBlastRadius::CanonicalFrame,
    );
    assert_rejected_counters(counters, 151, PhysicalDamageCause::SequenceMismatch);
}

#[test]
fn checksum_rejection_precedes_scope_identity_and_lsn_semantics() {
    for vector in [SEGMENT_HEX, LSN_HEX] {
        let mut bytes = literal(vector);
        bytes[116] ^= 1;
        let scope = scope(7, 1, 2, bytes.len() as u64);
        let (rejection, counters) = rejection(&bytes, scope);
        assert_damage(
            rejection,
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            0,
            bytes.len() as u64,
            None,
            PhysicalBlastRadius::CanonicalFrame,
        );
        assert_rejected_counters(counters, 151, PhysicalDamageCause::ChecksumMismatch);
    }
}

#[test]
fn checksum_coverage_distinguishes_combined_and_isolated_failures() {
    let cases = [
        (
            PAYLOAD_BYTE_HEX,
            0,
            151,
            None,
            PhysicalBlastRadius::CanonicalFrame,
        ),
        (
            PAYLOAD_CHECKSUM_HEX,
            84,
            32,
            Some(PhysicalFormatField::Checksum),
            PhysicalBlastRadius::DamagedRange,
        ),
        (
            FOOTER_CHECKSUM_HEX,
            119,
            32,
            Some(PhysicalFormatField::Checksum),
            PhysicalBlastRadius::DamagedRange,
        ),
    ];
    for (vector, offset, length, field, blast_radius) in cases {
        let bytes = literal(vector);
        let scope = scope(7, 1, 2, bytes.len() as u64);
        let (rejection, counters) = rejection(&bytes, scope);
        assert_damage(
            rejection,
            scope,
            PhysicalDamageCause::ChecksumMismatch,
            offset,
            length,
            field,
            blast_radius,
        );
        assert_rejected_counters(counters, 151, PhysicalDamageCause::ChecksumMismatch);
    }
}
