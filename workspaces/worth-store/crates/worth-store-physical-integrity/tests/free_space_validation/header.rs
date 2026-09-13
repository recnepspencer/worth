use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{PhysicalPageSizeClass, PhysicalRecordFormatDeclaration};
use worth_store_physical_integrity::{
    validate_free_space_header, FreeSpaceHeaderIntegrityValidation, PhysicalBlastRadius,
    PhysicalByteRange, PhysicalDamageCause, PhysicalFormatField, PhysicalIntegrityRejection,
    PhysicalIntegrityRejectionClass, PhysicalIntegrityVersionAxis, UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, assert_intact_counters, assert_rejected_counters, format, header_scope,
    header_scope_at, independent_crc32c, range, reseal, store, HEADER_COMPLETE_CRC32C,
    HEADER_LITERAL, HEADER_OFFSET,
};

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
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(HEADER_LITERAL);
    let (validation, counters) = validate_free_space_header(artifact, scope);
    let FreeSpaceHeaderIntegrityValidation::Intact(validated) = validation else {
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
    assert_eq!(validated.next_block(), 2);
    assert_eq!(validated.root().unwrap().block(), 1);
    assert_eq!(
        validated.complete_child_checksum().get(),
        HEADER_COMPLETE_CRC32C
    );
    assert!(validated.matches_input(artifact));
    let identical_copy = HEADER_LITERAL.to_vec();
    assert!(
        !validated.matches_input(UntrustedPhysicalArtifact::from_bounded_bytes(
            &identical_copy
        ))
    );
    assert_intact_counters(
        counters,
        PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
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
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_field_flip = HEADER_LITERAL.to_vec();
    checksum_field_flip[44] ^= 1;
    assert_header_damage(
        &checksum_field_flip,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut wrong_magic = HEADER_LITERAL.to_vec();
    wrong_magic[0] ^= 1;
    assert_header_damage(
        &wrong_magic,
        scope,
        PhysicalDamageCause::WrongMagic,
        range(scope, 0, 8),
        Some(PhysicalFormatField::Magic),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut wrong_kind = HEADER_LITERAL.to_vec();
    wrong_kind[8] = 10;
    reseal(&mut wrong_kind);
    assert_header_damage(
        &wrong_kind,
        scope,
        PhysicalDamageCause::FamilyMismatch,
        range(scope, 8, 1),
        Some(PhysicalFormatField::ArtifactFamily),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut length_lie = HEADER_LITERAL.to_vec();
    length_lie[24..28].copy_from_slice(&127_u32.to_le_bytes());
    reseal(&mut length_lie);
    assert_header_damage(
        &length_lie,
        scope,
        PhysicalDamageCause::FramingLengthMismatch,
        range(scope, 20, 8),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let truncated = &HEADER_LITERAL[..HEADER_LITERAL.len() - 7];
    assert_header_damage(
        truncated,
        scope,
        PhysicalDamageCause::Truncated,
        PhysicalByteRange::new(HEADER_OFFSET + 169, 7).unwrap(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut unsupported_schema = HEADER_LITERAL.to_vec();
    unsupported_schema[9] = 3;
    reseal(&mut unsupported_schema);
    assert_header_unsupported(
        &unsupported_schema,
        scope,
        PhysicalIntegrityVersionAxis::EnvelopeSchema,
        3,
    );

    let mut unsupported_format = HEADER_LITERAL.to_vec();
    unsupported_format[10..12].copy_from_slice(&2_u16.to_le_bytes());
    reseal(&mut unsupported_format);
    assert_header_unsupported(
        &unsupported_format,
        scope,
        PhysicalIntegrityVersionAxis::PhysicalFormat,
        2,
    );
}

fn assert_header_unsupported(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (FreeSpaceHeaderIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_header(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope)
    else {
        panic!("unsupported header validated");
    };
    let PhysicalIntegrityRejection::Unsupported(version) = rejection else {
        panic!("unsupported header collapsed into damage");
    };
    assert_eq!(version.axis(), axis);
    assert_eq!(version.observed(), observed);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
        bytes.len() as u64,
        PhysicalIntegrityRejectionClass::Unsupported,
    );
}

#[test]
fn header_format_generation_tree_root_and_complete_crc_substitution_fail_closed() {
    let scope = header_scope(store(7), HEADER_COMPLETE_CRC32C);

    let other_format = PhysicalRecordFormatDeclaration::builder()
        .page_size(PhysicalPageSizeClass::KiB32)
        .admit()
        .unwrap();
    let mut format_substitution = HEADER_LITERAL.to_vec();
    format_substitution[10..20].copy_from_slice(&other_format.canonical_identity_bytes());
    reseal(&mut format_substitution);
    assert_header_damage(
        &format_substitution,
        scope,
        PhysicalDamageCause::FormatMismatch,
        range(scope, 10, 10),
        Some(PhysicalFormatField::FormatDeclaration),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut generation = HEADER_LITERAL.to_vec();
    generation[28..36].copy_from_slice(&7_u64.to_le_bytes());
    generation[48..56].copy_from_slice(&7_u64.to_le_bytes());
    reseal(&mut generation);
    assert_header_damage(
        &generation,
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        range(scope, 28, 8),
        Some(PhysicalFormatField::PhysicalGeneration),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let mut tree = HEADER_LITERAL.to_vec();
    tree[56..64].copy_from_slice(&9_u64.to_le_bytes());
    reseal(&mut tree);
    assert_header_damage(
        &tree,
        scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        range(scope, 56, 8),
        Some(PhysicalFormatField::TreeIdentity),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let mut root = HEADER_LITERAL.to_vec();
    root[128..136].copy_from_slice(&2_u64.to_le_bytes());
    reseal(&mut root);
    assert_header_damage(
        &root,
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        range(scope, 112, 64),
        Some(PhysicalFormatField::ChildReference),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let wrong_crc_scope = header_scope(store(7), HEADER_COMPLETE_CRC32C ^ 1);
    assert_header_damage(
        HEADER_LITERAL,
        wrong_crc_scope,
        PhysicalDamageCause::ChecksumMismatch,
        wrong_crc_scope.byte_range(),
        Some(PhysicalFormatField::CompleteChildChecksum),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn header_validation_record_binds_store_and_range_without_claiming_embedded_store_bytes() {
    let left = header_scope(store(7), HEADER_COMPLETE_CRC32C);
    let right = header_scope(store(8), HEADER_COMPLETE_CRC32C);
    let shifted = header_scope_at(
        store(7),
        HEADER_COMPLETE_CRC32C,
        PhysicalByteRange::new(HEADER_OFFSET + 512, HEADER_LITERAL.len() as u64).unwrap(),
    );
    let shorter = header_scope_at(
        store(7),
        HEADER_COMPLETE_CRC32C,
        PhysicalByteRange::new(HEADER_OFFSET, HEADER_LITERAL.len() as u64 - 1).unwrap(),
    );
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(HEADER_LITERAL);
    let (FreeSpaceHeaderIntegrityValidation::Intact(left_validated), _) =
        validate_free_space_header(artifact, left)
    else {
        panic!("left scope rejected");
    };
    let (FreeSpaceHeaderIntegrityValidation::Intact(right_validated), _) =
        validate_free_space_header(artifact, right)
    else {
        panic!("right scope rejected");
    };
    let (FreeSpaceHeaderIntegrityValidation::Intact(shifted_validated), _) =
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
    let (FreeSpaceHeaderIntegrityValidation::Intact(changed_validated), _) =
        validate_free_space_header(
            UntrustedPhysicalArtifact::from_bounded_bytes(&changed),
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

fn assert_header_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    damaged_range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (FreeSpaceHeaderIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_header(UntrustedPhysicalArtifact::from_bounded_bytes(bytes), scope)
    else {
        panic!("damaged free-space header validated");
    };
    assert_damage(rejection, scope, cause, damaged_range, field, blast_radius);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
        bytes.len() as u64,
        PhysicalIntegrityRejectionClass::Damaged(cause),
    );
}
