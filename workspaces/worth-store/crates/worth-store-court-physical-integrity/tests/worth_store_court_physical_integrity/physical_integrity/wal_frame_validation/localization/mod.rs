use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
use {worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis};

use {crate::wal_frame_validation::support::assert_damage, crate::wal_frame_validation::support::assert_independent_frame_checksums, crate::wal_frame_validation::support::assert_rejected_counters, crate::wal_frame_validation::support::literal, crate::wal_frame_validation::support::rejection, crate::wal_frame_validation::support::scope, crate::wal_frame_validation::support::store, crate::wal_frame_validation::support::CLEAN_HEX, crate::wal_frame_validation::support::FOOTER_CHECKSUM_HEX, crate::wal_frame_validation::support::FRAME_OFFSET, crate::wal_frame_validation::support::GENERATION_HEX, crate::wal_frame_validation::support::HEADER_LENGTH_HEX, crate::wal_frame_validation::support::LENGTH_HEX, crate::wal_frame_validation::support::LSN_HEX, crate::wal_frame_validation::support::MAGIC_HEX, crate::wal_frame_validation::support::PAYLOAD_BYTE_HEX, crate::wal_frame_validation::support::PAYLOAD_CHECKSUM_HEX, crate::wal_frame_validation::support::SEGMENT_HEX, crate::wal_frame_validation::support::VERSION_HEX, crate::wal_frame_validation::support::ZERO_GENERATION_HEX, crate::wal_frame_validation::support::ZERO_SEGMENT_HEX};

#[test]
fn magic_and_family_denials_preserve_exact_scope() {
    let bytes = literal(MAGIC_HEX);
    let wal_scope = scope(7, 1, 2, bytes.len() as u64);
    let (magic_rejection, counters) = rejection(&bytes, wal_scope);
    assert_damage(
        magic_rejection,
        wal_scope,
        worth_store_physical_integrity::PhysicalDamageCause::WrongMagic,
        0,
        8,
        Some(worth_store_physical_integrity::PhysicalFormatField::Magic),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
    assert_rejected_counters(counters, 151, worth_store_physical_integrity::PhysicalDamageCause::WrongMagic);

    let clean = literal(CLEAN_HEX);
    let format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .admit()
        .unwrap();
    let selector_scope = worth_store_physical_integrity::PhysicalArtifactScope::current_root_selector(
        store(7),
        format,
        worth_store_physical_integrity::PhysicalByteRange::new(FRAME_OFFSET, clean.len() as u64).unwrap(),
    );
    let (rejection, _) = rejection(&clean, selector_scope);
    assert_damage(
        rejection,
        selector_scope,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        0,
        clean.len() as u64,
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn unsupported_version_is_not_collapsed_into_damage() {
    let bytes = literal(VERSION_HEX);
    assert_independent_frame_checksums(&bytes);
    let scope = scope(7, 1, 2, bytes.len() as u64);
    let (rejection, counters) = rejection(&bytes, scope);
    let worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(unsupported) = rejection else {
        panic!("unsupported WAL version must retain its posture");
    };
    assert_eq!(unsupported.scope(), scope);
    assert_eq!(unsupported.axis(), worth_store_physical_integrity::PhysicalIntegrityVersionAxis::WalFrame);
    assert_eq!(unsupported.observed(), 2);
    assert_eq!(counters.family(), worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::WalFrame);
    assert_eq!(
        counters.rejected_for(worth_store_physical_integrity::PhysicalIntegrityRejectionClass::Unsupported),
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
            worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
            offset,
            if offset == 10 { 2 } else { 8 },
            Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );
        assert_rejected_counters(counters, 151, worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch);
    }
}

#[test]
fn checksum_valid_segment_and_generation_substitution_are_distinct() {
    let cases = [
        (
            SEGMENT_HEX,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            12,
            worth_store_physical_integrity::PhysicalFormatField::SegmentIdentity,
        ),
        (
            GENERATION_HEX,
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            20,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
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
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
        );
        assert_rejected_counters(counters, 151, cause);
    }
}

#[test]
fn checksum_valid_zero_segment_and_generation_localize_separately() {
    let cases = [
        (
            ZERO_SEGMENT_HEX,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            12,
            worth_store_physical_integrity::PhysicalFormatField::SegmentIdentity,
        ),
        (
            ZERO_GENERATION_HEX,
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            20,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
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
            worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
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
        worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch,
        28,
        16,
        Some(worth_store_physical_integrity::PhysicalFormatField::WalLsnRange),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );
    assert_rejected_counters(counters, 151, worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch);
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
            worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
            0,
            bytes.len() as u64,
            None,
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        );
        assert_rejected_counters(counters, 151, worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch);
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
            worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
        ),
        (
            PAYLOAD_CHECKSUM_HEX,
            84,
            32,
            Some(worth_store_physical_integrity::PhysicalFormatField::Checksum),
            worth_store_physical_integrity::PhysicalBlastRadius::DamagedRange,
        ),
        (
            FOOTER_CHECKSUM_HEX,
            119,
            32,
            Some(worth_store_physical_integrity::PhysicalFormatField::Checksum),
            worth_store_physical_integrity::PhysicalBlastRadius::DamagedRange,
        ),
    ];
    for (vector, offset, length, field, blast_radius) in cases {
        let bytes = literal(vector);
        let scope = scope(7, 1, 2, bytes.len() as u64);
        let (rejection, counters) = rejection(&bytes, scope);
        assert_damage(
            rejection,
            scope,
            worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
            offset,
            length,
            field,
            blast_radius,
        );
        assert_rejected_counters(counters, 151, worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch);
    }
}
