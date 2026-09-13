use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{FreeSpaceBlockReference, FreeSpaceKey, RecordAllocationClass};
use worth_store_physical_integrity::{
    validate_free_space_membership_block, FreeSpaceMembershipBlockIntegrityValidation,
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejectionClass, PhysicalIntegrityVersionAxis,
    UntrustedPhysicalArtifact,
};

use super::support::{
    assert_damage, assert_intact_counters, assert_rejected_counters, first_key, format,
    independent_crc32c, last_key, membership_reference, membership_scope, membership_scope_at,
    range, reseal, store, MEMBERSHIP_COMPLETE_CRC32C, MEMBERSHIP_LITERAL, MEMBERSHIP_OFFSET,
};

#[test]
fn independent_literal_membership_seals_parent_crc_range_and_leaf_projection() {
    assert_eq!(
        independent_crc32c(&[MEMBERSHIP_LITERAL]),
        MEMBERSHIP_COMPLETE_CRC32C
    );
    assert_eq!(
        independent_crc32c(&[&MEMBERSHIP_LITERAL[..44], &MEMBERSHIP_LITERAL[48..]]),
        u32::from_le_bytes(MEMBERSHIP_LITERAL[44..48].try_into().unwrap())
    );
    let scope = membership_scope(store(7), membership_reference(MEMBERSHIP_COMPLETE_CRC32C));
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(MEMBERSHIP_LITERAL);
    let (validation, counters) = validate_free_space_membership_block(artifact, scope);
    let FreeSpaceMembershipBlockIntegrityValidation::Intact(validated) = validation else {
        panic!("literal free-space membership block rejected");
    };

    assert_eq!(validated.scope(), scope);
    assert_eq!(validated.identity().tree().get(), 8);
    assert_eq!(
        validated.reference(),
        membership_reference(MEMBERSHIP_COMPLETE_CRC32C)
    );
    assert_eq!(validated.record_format(), format());
    assert_eq!(validated.generation(), 6);
    assert_eq!(validated.block_identity(), 1);
    assert_eq!(validated.level(), 0);
    let entries = validated.entries().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].class(), RecordAllocationClass::InlinePage);
    assert_eq!(entries[0].owner(), 7);
    assert_eq!(entries[1].class(), RecordAllocationClass::Extent);
    assert_eq!(entries[1].owner(), 5);
    assert!(validated.children().is_none());
    assert!(validated.matches_input(artifact));
    let identical_copy = MEMBERSHIP_LITERAL.to_vec();
    assert!(
        !validated.matches_input(UntrustedPhysicalArtifact::from_bounded_bytes(
            &identical_copy
        ))
    );
    assert_intact_counters(
        counters,
        PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
        MEMBERSHIP_LITERAL.len() as u64,
    );
    assert!(validated.into_validation_record().matches_scope(scope));
}

#[test]
fn membership_framing_checksum_version_kind_length_and_truncation_fail_closed() {
    let scope = membership_scope(store(7), membership_reference(MEMBERSHIP_COMPLETE_CRC32C));

    let mut checksum_damage = MEMBERSHIP_LITERAL.to_vec();
    checksum_damage[96] ^= 1;
    assert_membership_damage(
        &checksum_damage,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_field_flip = MEMBERSHIP_LITERAL.to_vec();
    checksum_field_flip[44] ^= 1;
    assert_membership_damage(
        &checksum_field_flip,
        scope,
        PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut wrong_kind = MEMBERSHIP_LITERAL.to_vec();
    wrong_kind[8] = 7;
    reseal(&mut wrong_kind);
    assert_membership_damage(
        &wrong_kind,
        scope,
        PhysicalDamageCause::FamilyMismatch,
        range(scope, 8, 1),
        Some(PhysicalFormatField::ArtifactFamily),
        PhysicalBlastRadius::CompleteArtifact,
    );

    let mut length_lie = MEMBERSHIP_LITERAL.to_vec();
    length_lie[24..28].copy_from_slice(&119_u32.to_le_bytes());
    reseal(&mut length_lie);
    assert_membership_damage(
        &length_lie,
        scope,
        PhysicalDamageCause::FramingLengthMismatch,
        range(scope, 20, 8),
        Some(PhysicalFormatField::EncodedLength),
        PhysicalBlastRadius::CanonicalFrame,
    );

    let truncated = &MEMBERSHIP_LITERAL[..MEMBERSHIP_LITERAL.len() - 9];
    assert_membership_damage(
        truncated,
        scope,
        PhysicalDamageCause::Truncated,
        PhysicalByteRange::new(MEMBERSHIP_OFFSET + 159, 9).unwrap(),
        None,
        PhysicalBlastRadius::CanonicalFrame,
    );

    let mut unsupported_schema = MEMBERSHIP_LITERAL.to_vec();
    unsupported_schema[9] = 3;
    reseal(&mut unsupported_schema);
    assert_membership_unsupported(
        &unsupported_schema,
        scope,
        PhysicalIntegrityVersionAxis::EnvelopeSchema,
        3,
    );

    let mut unsupported_format = MEMBERSHIP_LITERAL.to_vec();
    unsupported_format[10..12].copy_from_slice(&2_u16.to_le_bytes());
    reseal(&mut unsupported_format);
    assert_membership_unsupported(
        &unsupported_format,
        scope,
        PhysicalIntegrityVersionAxis::PhysicalFormat,
        2,
    );
}

fn assert_membership_unsupported(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    axis: PhysicalIntegrityVersionAxis,
    observed: u32,
) {
    let (FreeSpaceMembershipBlockIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_membership_block(
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            scope,
        )
    else {
        panic!("unsupported membership block validated");
    };
    let worth_store_physical_integrity::PhysicalIntegrityRejection::Unsupported(version) =
        rejection
    else {
        panic!("unsupported format collapsed into damage");
    };
    assert_eq!(version.axis(), axis);
    assert_eq!(version.observed(), observed);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
        bytes.len() as u64,
        PhysicalIntegrityRejectionClass::Unsupported,
    );
}

#[test]
fn membership_tree_generation_block_reference_range_and_parent_crc_are_not_substitutable() {
    let scope = membership_scope(store(7), membership_reference(MEMBERSHIP_COMPLETE_CRC32C));

    let mut tree = MEMBERSHIP_LITERAL.to_vec();
    tree[48..56].copy_from_slice(&9_u64.to_le_bytes());
    reseal(&mut tree);
    assert_membership_damage(
        &tree,
        scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        range(scope, 48, 8),
        Some(PhysicalFormatField::TreeIdentity),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let mut generation = MEMBERSHIP_LITERAL.to_vec();
    generation[72..80].copy_from_slice(&7_u64.to_le_bytes());
    reseal(&mut generation);
    assert_membership_damage(
        &generation,
        scope,
        PhysicalDamageCause::PhysicalGenerationMismatch,
        range(scope, 72, 8),
        Some(PhysicalFormatField::PhysicalGeneration),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let mut block = MEMBERSHIP_LITERAL.to_vec();
    block[28..36].copy_from_slice(&2_u64.to_le_bytes());
    block[56..64].copy_from_slice(&2_u64.to_le_bytes());
    reseal(&mut block);
    assert_membership_damage(
        &block,
        scope,
        PhysicalDamageCause::ArtifactIdentityMismatch,
        range(scope, 28, 8),
        Some(PhysicalFormatField::BlockIdentity),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let foreign_level =
        FreeSpaceBlockReference::new(6, 1, 1, MEMBERSHIP_COMPLETE_CRC32C, first_key(), last_key())
            .unwrap();
    let level_scope = membership_scope(store(7), foreign_level);
    assert_membership_damage(
        MEMBERSHIP_LITERAL,
        level_scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        range(level_scope, 64, 2),
        Some(PhysicalFormatField::ChildReference),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let foreign_last = FreeSpaceKey::new(RecordAllocationClass::Extent, 6).unwrap();
    let foreign_range = FreeSpaceBlockReference::new(
        6,
        1,
        0,
        MEMBERSHIP_COMPLETE_CRC32C,
        first_key(),
        foreign_last,
    )
    .unwrap();
    let range_scope = membership_scope(store(7), foreign_range);
    assert_membership_damage(
        MEMBERSHIP_LITERAL,
        range_scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        range(range_scope, 88, 80),
        Some(PhysicalFormatField::MembershipRange),
        PhysicalBlastRadius::ReachableSubtree,
    );

    let crc_scope = membership_scope(
        store(7),
        membership_reference(MEMBERSHIP_COMPLETE_CRC32C ^ 1),
    );
    assert_membership_damage(
        MEMBERSHIP_LITERAL,
        crc_scope,
        PhysicalDamageCause::ChecksumMismatch,
        crc_scope.byte_range(),
        Some(PhysicalFormatField::CompleteChildChecksum),
        PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn leaf_ordering_is_rejected_before_parent_checksum_agreement() {
    let leaf_scope = membership_scope(store(7), membership_reference(MEMBERSHIP_COMPLETE_CRC32C));
    let mut reordered_leaf = MEMBERSHIP_LITERAL.to_vec();
    reordered_leaf[128] = 1;
    reordered_leaf[136..144].copy_from_slice(&6_u64.to_le_bytes());
    reseal(&mut reordered_leaf);
    assert_membership_damage(
        &reordered_leaf,
        leaf_scope,
        PhysicalDamageCause::SequenceMismatch,
        range(leaf_scope, 128, 40),
        Some(PhysicalFormatField::MembershipRange),
        PhysicalBlastRadius::ReachableSubtree,
    );
}

#[test]
fn membership_validation_record_binds_store_without_reclassifying_allocator_truth() {
    let reference = membership_reference(MEMBERSHIP_COMPLETE_CRC32C);
    let left = membership_scope(store(7), reference);
    let right = membership_scope(store(8), reference);
    let shifted = membership_scope_at(
        store(7),
        reference,
        PhysicalByteRange::new(MEMBERSHIP_OFFSET + 512, MEMBERSHIP_LITERAL.len() as u64).unwrap(),
    );
    let shorter = membership_scope_at(
        store(7),
        reference,
        PhysicalByteRange::new(MEMBERSHIP_OFFSET, MEMBERSHIP_LITERAL.len() as u64 - 1).unwrap(),
    );
    let artifact = UntrustedPhysicalArtifact::from_bounded_bytes(MEMBERSHIP_LITERAL);
    let (FreeSpaceMembershipBlockIntegrityValidation::Intact(left_validated), _) =
        validate_free_space_membership_block(artifact, left)
    else {
        panic!("left scope rejected");
    };
    let (FreeSpaceMembershipBlockIntegrityValidation::Intact(right_validated), _) =
        validate_free_space_membership_block(artifact, right)
    else {
        panic!("right scope rejected");
    };
    let (FreeSpaceMembershipBlockIntegrityValidation::Intact(shifted_validated), _) =
        validate_free_space_membership_block(artifact, shifted)
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

    let mut changed = MEMBERSHIP_LITERAL.to_vec();
    changed[104..112].copy_from_slice(&5_u64.to_le_bytes());
    reseal(&mut changed);
    let changed_crc = independent_crc32c(&[&changed]);
    let changed_scope = membership_scope(store(7), membership_reference(changed_crc));
    let (FreeSpaceMembershipBlockIntegrityValidation::Intact(changed_validated), _) =
        validate_free_space_membership_block(
            UntrustedPhysicalArtifact::from_bounded_bytes(&changed),
            changed_scope,
        )
    else {
        panic!("changed byte range rejected");
    };
    assert_ne!(
        left_record.byte_range_digest(),
        changed_validated
            .into_validation_record()
            .byte_range_digest()
    );
}

fn assert_membership_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    damaged_range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (FreeSpaceMembershipBlockIntegrityValidation::Rejected(rejection), counters) =
        validate_free_space_membership_block(
            UntrustedPhysicalArtifact::from_bounded_bytes(bytes),
            scope,
        )
    else {
        panic!("damaged free-space membership block validated");
    };
    assert_damage(rejection, scope, cause, damaged_range, field, blast_radius);
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
        bytes.len() as u64,
        PhysicalIntegrityRejectionClass::Damaged(cause),
    );
}
