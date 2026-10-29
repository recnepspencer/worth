#[allow(unused_imports)] use crate::free_space_validation::header::*;
use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
use {worth_store_physical_format::PhysicalPageSizeClass, worth_store_physical_format::PhysicalRecordFormatDeclaration};
use {worth_store_physical_integrity::validate_free_space_header, worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityRejectionClass, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

use {crate::support::free_space_validation::assert_damage, crate::support::free_space_validation::assert_intact_counters, crate::support::free_space_validation::assert_rejected_counters, crate::support::free_space_validation::format, crate::support::free_space_validation::header_scope, crate::support::free_space_validation::header_scope_at, crate::support::free_space_validation::independent_crc32c, crate::support::free_space_validation::range, crate::support::free_space_validation::reseal, crate::support::free_space_validation::store, crate::support::free_space_validation::HEADER_COMPLETE_CRC32C, crate::support::free_space_validation::HEADER_LITERAL, crate::support::free_space_validation::HEADER_OFFSET};

#[test]
fn independent_literal_header_seals_root_issued_identity_and_owner_projection() {
    assert_eq!(
        independent_crc32c(&[HEADER_LITERAL]),
        HEADER_COMPLETE_CRC32C
    );
    assert_eq!(
        independent_crc32c(&[&HEADER_LITERAL[..44], &HEADER_LITERAL[48..]]),
        u32::from_le_bytes(HEADER_LITERAL[44..48].try_into().unwrap())
    );
    let scope = header_scope(store(7), HEADER_COMPLETE_CRC32C);
    let artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(HEADER_LITERAL);
    let (validation, counters) = validate_free_space_header(artifact, scope);
    let worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Intact(validated) = validation else {
        panic!("literal free-space header rejected");
    };

    assert_eq!(validated.scope(), scope);
    assert_eq!(validated.identity().generation().get(), 6);
    assert_eq!(validated.identity().tree().get(), 8);
    assert_eq!(validated.record_format(), format());
    assert_eq!(validated.node_capacity(), 2);
    assert_eq!(validated.segment_page_capacity(), 4);
    assert_eq!(validated.entry_count(), 2);
    assert_eq!(validated.next_segment(), 8);
    assert_eq!(validated.next_page(), 10);
    assert_eq!(validated.next_extent(), 5);
    assert_eq!(validated.next_arena(), 6);
    assert_eq!(validated.arena_capacity(), 64 * 1024 * 1024);
    assert_eq!(validated.arena_alignment(), 16 * 1024);
    assert_eq!(validated.next_block(), 2);
    assert_eq!(validated.root().unwrap().block(), 1);
    assert_eq!(
        validated.complete_child_checksum().get(),
        HEADER_COMPLETE_CRC32C
    );
    assert!(validated.matches_input(artifact));
    let identical_copy = HEADER_LITERAL.to_vec();
    assert!(
        !validated.matches_input(worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &identical_copy
        ))
    );
    assert_intact_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
        HEADER_LITERAL.len() as u64,
    );
    assert!(validated.into_validation_record().matches_scope(scope));
}

#[test]
fn header_framing_version_kind_length_and_truncation_are_precisely_localized() {
    let scope = header_scope(store(7), HEADER_COMPLETE_CRC32C);

    let mut covered_byte_flip = HEADER_LITERAL.to_vec();
    covered_byte_flip[80] ^= 1;
    assert_header_damage(
        &covered_byte_flip,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_field_flip = HEADER_LITERAL.to_vec();
    checksum_field_flip[44] ^= 1;
    assert_header_damage(
        &checksum_field_flip,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut wrong_magic = HEADER_LITERAL.to_vec();
    wrong_magic[0] ^= 1;
    assert_header_damage(
        &wrong_magic,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::WrongMagic,
        range(scope, 0, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::Magic),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut wrong_kind = HEADER_LITERAL.to_vec();
    wrong_kind[8] = 10;
    reseal(&mut wrong_kind);
    assert_header_damage(
        &wrong_kind,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        range(scope, 8, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::ArtifactFamily),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut length_lie = HEADER_LITERAL.to_vec();
    length_lie[24..28].copy_from_slice(&127_u32.to_le_bytes());
    reseal(&mut length_lie);
    assert_header_damage(
        &length_lie,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        range(scope, 20, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let truncated = &HEADER_LITERAL[..HEADER_LITERAL.len() - 7];
    assert_header_damage(
        truncated,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::Truncated,
        worth_store_physical_integrity::PhysicalByteRange::new(HEADER_OFFSET + 209, 7).unwrap(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut unsupported_schema = HEADER_LITERAL.to_vec();
    unsupported_schema[9] = 3;
    reseal(&mut unsupported_schema);
    assert_header_unsupported(
        &unsupported_schema,
        scope,
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::EnvelopeSchema,
        3,
    );

    let mut unsupported_format = HEADER_LITERAL.to_vec();
    unsupported_format[10..12].copy_from_slice(&3_u16.to_le_bytes());
    reseal(&mut unsupported_format);
    assert_header_unsupported(
        &unsupported_format,
        scope,
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat,
        3,
    );
}

#[test]
fn header_format_generation_tree_root_and_complete_crc_substitution_fail_closed() {
    let scope = header_scope(store(7), HEADER_COMPLETE_CRC32C);

    let other_format = worth_store_physical_format::PhysicalRecordFormatDeclaration::builder()
        .page_size(worth_store_physical_format::PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    let mut format_substitution = HEADER_LITERAL.to_vec();
    format_substitution[10..20].copy_from_slice(&other_format.canonical_identity_bytes());
    reseal(&mut format_substitution);
    assert_header_damage(
        &format_substitution,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FormatMismatch,
        range(scope, 10, 10),
        Some(worth_store_physical_integrity::PhysicalFormatField::FormatDeclaration),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut generation = HEADER_LITERAL.to_vec();
    generation[28..36].copy_from_slice(&7_u64.to_le_bytes());
    generation[48..56].copy_from_slice(&7_u64.to_le_bytes());
    reseal(&mut generation);
    assert_header_damage(
        &generation,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
        range(scope, 28, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let mut tree = HEADER_LITERAL.to_vec();
    tree[56..64].copy_from_slice(&9_u64.to_le_bytes());
    reseal(&mut tree);
    assert_header_damage(
        &tree,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
        range(scope, 56, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::TreeIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let mut root = HEADER_LITERAL.to_vec();
    root[128..136].copy_from_slice(&2_u64.to_le_bytes());
    reseal(&mut root);
    assert_header_damage(
        &root,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        range(scope, 112, 80),
        Some(worth_store_physical_integrity::PhysicalFormatField::ChildReference),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let wrong_crc_scope = header_scope(store(7), HEADER_COMPLETE_CRC32C ^ 1);
    assert_header_damage(
        HEADER_LITERAL,
        wrong_crc_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        wrong_crc_scope.byte_range(),
        Some(worth_store_physical_integrity::PhysicalFormatField::CompleteChildChecksum),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn header_validation_record_binds_store_and_range_without_claiming_embedded_store_bytes() {
    let left = header_scope(store(7), HEADER_COMPLETE_CRC32C);
    let right = header_scope(store(8), HEADER_COMPLETE_CRC32C);
    let shifted = header_scope_at(
        store(7),
        HEADER_COMPLETE_CRC32C,
        worth_store_physical_integrity::PhysicalByteRange::new(HEADER_OFFSET + 512, HEADER_LITERAL.len() as u64).unwrap(),
    );
    let shorter = header_scope_at(
        store(7),
        HEADER_COMPLETE_CRC32C,
        worth_store_physical_integrity::PhysicalByteRange::new(HEADER_OFFSET, HEADER_LITERAL.len() as u64 - 1).unwrap(),
    );
    let artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(HEADER_LITERAL);
    let (worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Intact(left_validated), _) =
        validate_free_space_header(artifact, left)
    else {
        panic!("left scope rejected");
    };
    let (worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Intact(right_validated), _) =
        validate_free_space_header(artifact, right)
    else {
        panic!("right scope rejected");
    };
    let (worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Intact(shifted_validated), _) =
        validate_free_space_header(artifact, shifted)
    else {
        panic!("shifted scope rejected");
    };
    let left_record = left_validated.into_validation_record();
    let right_record = right_validated.into_validation_record();
    let shifted_record = shifted_validated.into_validation_record();
    assert_ne!(
        left_record.exact_scope_digest(),
        right_record.exact_scope_digest()
    );
    assert_eq!(
        left_record.byte_range_digest(),
        right_record.byte_range_digest()
    );
    assert_ne!(
        left_record.exact_scope_digest(),
        shifted_record.exact_scope_digest()
    );
    assert_eq!(
        left_record.byte_range_digest(),
        shifted_record.byte_range_digest()
    );
    assert!(!left_record.matches_scope(right));
    assert!(!left_record.matches_scope(shifted));
    assert!(!left_record.matches_scope(shorter));

    let mut changed = HEADER_LITERAL.to_vec();
    changed[88..96].copy_from_slice(&11_u64.to_le_bytes());
    reseal(&mut changed);
    let changed_crc = independent_crc32c(&[&changed]);
    let changed_scope = header_scope(store(7), changed_crc);
    let (worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Intact(changed_validated), _) =
        validate_free_space_header(
            worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&changed),
            changed_scope,
        )
    else {
        panic!("changed byte range rejected");
    };
    let changed_record = changed_validated.into_validation_record();
    assert_ne!(
        left_record.byte_range_digest(),
        changed_record.byte_range_digest()
    );
}
