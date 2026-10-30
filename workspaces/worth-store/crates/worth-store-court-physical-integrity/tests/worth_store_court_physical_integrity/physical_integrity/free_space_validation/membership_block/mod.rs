#[allow(unused_imports)]
use crate::free_space_validation::membership_block::*;

#[test]
fn membership_framing_checksum_version_kind_length_and_truncation_fail_closed() {
    let scope = membership_scope(store(7), membership_reference(MEMBERSHIP_COMPLETE_CRC32C));

    let mut checksum_damage = MEMBERSHIP_LITERAL.to_vec();
    checksum_damage[96] ^= 1;
    assert_membership_damage(
        &checksum_damage,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut checksum_field_flip = MEMBERSHIP_LITERAL.to_vec();
    checksum_field_flip[44] ^= 1;
    assert_membership_damage(
        &checksum_field_flip,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        scope.byte_range(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut wrong_kind = MEMBERSHIP_LITERAL.to_vec();
    wrong_kind[8] = 7;
    reseal(&mut wrong_kind);
    assert_membership_damage(
        &wrong_kind,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FamilyMismatch,
        range(scope, 8, 1),
        Some(worth_store_physical_integrity::PhysicalFormatField::ArtifactFamily),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );

    let mut length_lie = MEMBERSHIP_LITERAL.to_vec();
    length_lie[24..28].copy_from_slice(&119_u32.to_le_bytes());
    reseal(&mut length_lie);
    assert_membership_damage(
        &length_lie,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::FramingLengthMismatch,
        range(scope, 20, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::EncodedLength),
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let truncated = &MEMBERSHIP_LITERAL[..MEMBERSHIP_LITERAL.len() - 9];
    assert_membership_damage(
        truncated,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::Truncated,
        worth_store_physical_integrity::PhysicalByteRange::new(MEMBERSHIP_OFFSET + 159, 9).unwrap(),
        None,
        worth_store_physical_integrity::PhysicalBlastRadius::CanonicalFrame,
    );

    let mut unsupported_schema = MEMBERSHIP_LITERAL.to_vec();
    unsupported_schema[9] = 3;
    reseal(&mut unsupported_schema);
    assert_membership_unsupported(
        &unsupported_schema,
        scope,
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::EnvelopeSchema,
        3,
    );

    let mut unsupported_format = MEMBERSHIP_LITERAL.to_vec();
    unsupported_format[10..12].copy_from_slice(&3_u16.to_le_bytes());
    reseal(&mut unsupported_format);
    assert_membership_unsupported(
        &unsupported_format,
        scope,
        worth_store_physical_integrity::PhysicalIntegrityVersionAxis::PhysicalFormat,
        3,
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
        worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
        range(scope, 48, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::TreeIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let mut generation = MEMBERSHIP_LITERAL.to_vec();
    generation[72..80].copy_from_slice(&7_u64.to_le_bytes());
    reseal(&mut generation);
    assert_membership_damage(
        &generation,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
        range(scope, 72, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let mut block = MEMBERSHIP_LITERAL.to_vec();
    block[28..36].copy_from_slice(&2_u64.to_le_bytes());
    block[56..64].copy_from_slice(&2_u64.to_le_bytes());
    reseal(&mut block);
    assert_membership_damage(
        &block,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
        range(scope, 28, 8),
        Some(worth_store_physical_integrity::PhysicalFormatField::BlockIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let foreign_level = worth_store_physical_format::FreeSpaceBlockReference::new(
        6,
        1,
        1,
        MEMBERSHIP_COMPLETE_CRC32C,
        first_key(),
        last_key(),
    )
    .unwrap();
    let level_scope = membership_scope(store(7), foreign_level);
    assert_membership_damage(
        MEMBERSHIP_LITERAL,
        level_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        range(level_scope, 64, 2),
        Some(worth_store_physical_integrity::PhysicalFormatField::ChildReference),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let foreign_last = worth_store_physical_format::FreeSpaceKey::arena(
        worth_store_physical_format::ExtentArenaId::new(6).unwrap(),
        100,
    );
    let foreign_range = worth_store_physical_format::FreeSpaceBlockReference::new(
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
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        range(range_scope, 88, 80),
        Some(worth_store_physical_integrity::PhysicalFormatField::MembershipRange),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let wrong_offset = worth_store_physical_format::FreeSpaceBlockReference::new(
        6,
        1,
        0,
        MEMBERSHIP_COMPLETE_CRC32C,
        first_key(),
        worth_store_physical_format::FreeSpaceKey::arena(
            worth_store_physical_format::ExtentArenaId::new(5).unwrap(),
            101,
        ),
    )
    .unwrap();
    let offset_scope = membership_scope(store(7), wrong_offset);
    assert_membership_damage(
        MEMBERSHIP_LITERAL,
        offset_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        range(offset_scope, 88, 80),
        Some(worth_store_physical_integrity::PhysicalFormatField::MembershipRange),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let crc_scope = membership_scope(
        store(7),
        membership_reference(MEMBERSHIP_COMPLETE_CRC32C ^ 1),
    );
    assert_membership_damage(
        MEMBERSHIP_LITERAL,
        crc_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChecksumMismatch,
        crc_scope.byte_range(),
        Some(worth_store_physical_integrity::PhysicalFormatField::CompleteChildChecksum),
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
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
        worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch,
        range(leaf_scope, 128, 40),
        Some(worth_store_physical_integrity::PhysicalFormatField::MembershipRange),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
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
        worth_store_physical_integrity::PhysicalByteRange::new(
            MEMBERSHIP_OFFSET + 512,
            MEMBERSHIP_LITERAL.len() as u64,
        )
        .unwrap(),
    );
    let shorter = membership_scope_at(
        store(7),
        reference,
        worth_store_physical_integrity::PhysicalByteRange::new(
            MEMBERSHIP_OFFSET,
            MEMBERSHIP_LITERAL.len() as u64 - 1,
        )
        .unwrap(),
    );
    let artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
        MEMBERSHIP_LITERAL,
    );
    let (
        worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation::Intact(
            left_validated,
        ),
        _,
    ) = validate_free_space_membership_block(artifact, left)
    else {
        panic!("left scope rejected");
    };
    let (
        worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation::Intact(
            right_validated,
        ),
        _,
    ) = validate_free_space_membership_block(artifact, right)
    else {
        panic!("right scope rejected");
    };
    let (
        worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation::Intact(
            shifted_validated,
        ),
        _,
    ) = validate_free_space_membership_block(artifact, shifted)
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
    let (
        worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation::Intact(
            changed_validated,
        ),
        _,
    ) = validate_free_space_membership_block(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&changed),
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
