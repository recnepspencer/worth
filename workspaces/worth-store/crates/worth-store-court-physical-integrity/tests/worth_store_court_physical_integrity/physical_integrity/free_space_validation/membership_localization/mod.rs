#[allow(unused_imports)]
use crate::free_space_validation::membership_localization::*;

#[test]
fn intact_branch_seals_children_and_input_incarnation() {
    let (bytes, scope, expected) = branch_fixture();
    let artifact =
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(&bytes);
    let (validation, counters) = validate_free_space_membership_block(artifact, scope);
    let worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation::Intact(
        validated,
    ) = validation
    else {
        panic!("clean branch rejected");
    };
    assert_eq!(validated.scope(), scope);
    assert_eq!(validated.level(), 1);
    assert_eq!(validated.children().unwrap(), expected.as_slice());
    assert!(validated.entries().is_none());
    assert!(validated.matches_input(artifact));
    let identical_copy = bytes.clone();
    assert!(!validated.matches_input(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &identical_copy
        )
    ));
    assert!(validated.into_validation_record().matches_scope(scope));
    assert_intact_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
        bytes.len() as u64,
    );
}

#[test]
fn branch_child_level_generation_order_and_shape_localize_the_offending_child() {
    let (single_child, single_scope) = one_child_branch_fixture();
    for (offset, value) in [(104, 1_u64), (88, 7)] {
        let mut bytes = single_child.clone();
        if offset == 104 {
            bytes[offset..offset + 2].copy_from_slice(&(value as u16).to_le_bytes());
        } else {
            bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        }
        reseal(&mut bytes);
        assert_membership_localization(
            &bytes,
            single_scope,
            worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
            88,
            72,
            worth_store_physical_integrity::PhysicalFormatField::ChildReference,
            worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
        );
    }

    let (clean, scope, _) = branch_fixture();
    let mut reordered = clean.clone();
    reordered[192..200].copy_from_slice(&2_u64.to_le_bytes());
    reseal(&mut reordered);
    assert_membership_localization(
        &reordered,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::SequenceMismatch,
        160,
        72,
        worth_store_physical_integrity::PhysicalFormatField::MembershipRange,
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let mut invalid_second = clean;
    invalid_second[178] = 1;
    reseal(&mut invalid_second);
    assert_membership_localization(
        &invalid_second,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        160,
        72,
        worth_store_physical_integrity::PhysicalFormatField::ChildReference,
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
}

#[test]
fn malformed_second_leaf_entry_is_not_widened_to_the_whole_body() {
    let scope = membership_scope(store(7), membership_reference(MEMBERSHIP_COMPLETE_CRC32C));
    let mut bytes = MEMBERSHIP_LITERAL.to_vec();
    bytes[129] = 1;
    reseal(&mut bytes);
    assert_membership_localization(
        &bytes,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::MalformedStructure,
        128,
        40,
        worth_store_physical_integrity::PhysicalFormatField::Payload,
        worth_store_physical_integrity::PhysicalBlastRadius::CompleteArtifact,
    );
}

#[test]
fn membership_envelope_payload_and_zero_identities_blame_the_substituted_field() {
    let scope = membership_scope(store(7), membership_reference(MEMBERSHIP_COMPLETE_CRC32C));
    for (offset, value, cause, field) in [
        (
            28,
            2_u64,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            worth_store_physical_integrity::PhysicalFormatField::BlockIdentity,
        ),
        (
            56,
            2,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            worth_store_physical_integrity::PhysicalFormatField::BlockIdentity,
        ),
        (
            72,
            0,
            worth_store_physical_integrity::PhysicalDamageCause::PhysicalGenerationMismatch,
            worth_store_physical_integrity::PhysicalFormatField::PhysicalGeneration,
        ),
        (
            48,
            0,
            worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
            worth_store_physical_integrity::PhysicalFormatField::TreeIdentity,
        ),
    ] {
        let mut bytes = MEMBERSHIP_LITERAL.to_vec();
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        reseal(&mut bytes);
        assert_membership_localization(
            &bytes,
            scope,
            cause,
            offset as u64,
            8,
            field,
            worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
        );
    }

    let mut coherent = MEMBERSHIP_LITERAL.to_vec();
    coherent[28..36].copy_from_slice(&2_u64.to_le_bytes());
    coherent[56..64].copy_from_slice(&2_u64.to_le_bytes());
    reseal(&mut coherent);
    assert_membership_localization(
        &coherent,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ArtifactIdentityMismatch,
        28,
        8,
        worth_store_physical_integrity::PhysicalFormatField::BlockIdentity,
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
}

#[test]
fn membership_level_substitution_is_distinct_from_a_true_kind_mutation() {
    let leaf_scope = membership_scope(store(7), membership_reference(MEMBERSHIP_COMPLETE_CRC32C));
    let mut leaf_level = MEMBERSHIP_LITERAL.to_vec();
    leaf_level[64..66].copy_from_slice(&1_u16.to_le_bytes());
    reseal(&mut leaf_level);
    assert_membership_localization(
        &leaf_level,
        leaf_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        64,
        2,
        worth_store_physical_integrity::PhysicalFormatField::ChildReference,
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let (branch, branch_scope, _) = branch_fixture();
    let mut branch_level = branch;
    branch_level[64..66].fill(0);
    reseal(&mut branch_level);
    assert_membership_localization(
        &branch_level,
        branch_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        64,
        2,
        worth_store_physical_integrity::PhysicalFormatField::ChildReference,
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let mut wrong_kind = MEMBERSHIP_LITERAL.to_vec();
    wrong_kind[68] = 2;
    reseal(&mut wrong_kind);
    assert_membership_localization(
        &wrong_kind,
        leaf_scope,
        worth_store_physical_integrity::PhysicalDamageCause::RecordKindMismatch,
        68,
        1,
        worth_store_physical_integrity::PhysicalFormatField::MembershipKind,
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let leaf_reference = membership_reference(MEMBERSHIP_COMPLETE_CRC32C);
    let leaf_as_branch = worth_store_physical_format::FreeSpaceBlockReference::new(
        leaf_reference.generation(),
        leaf_reference.block(),
        1,
        leaf_reference.checksum(),
        leaf_reference.first(),
        leaf_reference.last(),
    )
    .unwrap();
    let leaf_as_branch_scope = membership_scope(store(7), leaf_as_branch);
    assert_membership_localization(
        MEMBERSHIP_LITERAL,
        leaf_as_branch_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        64,
        2,
        worth_store_physical_integrity::PhysicalFormatField::ChildReference,
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );

    let (branch, branch_scope, _) = branch_fixture();
    let branch_reference = branch_scope
        .free_space_membership_block_identity()
        .unwrap()
        .reference();
    let branch_as_leaf = worth_store_physical_format::FreeSpaceBlockReference::new(
        branch_reference.generation(),
        branch_reference.block(),
        0,
        branch_reference.checksum(),
        branch_reference.first(),
        branch_reference.last(),
    )
    .unwrap();
    let branch_as_leaf_scope =
        worth_store_physical_integrity::PhysicalArtifactScope::free_space_membership_block(
            store(7),
            format(),
            worth_store_physical_format::FreeSpaceMembershipBlockScopeIdentity::new(
                worth_store_physical_format::PhysicalTreeIdentity::new(8).unwrap(),
                branch_as_leaf,
            ),
            branch_scope.byte_range(),
        );
    assert_membership_localization(
        &branch,
        branch_as_leaf_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        64,
        2,
        worth_store_physical_integrity::PhysicalFormatField::ChildReference,
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
}
