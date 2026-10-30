#[allow(unused_imports)]
use crate::free_space_validation::membership_block::*;

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
    let artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
        MEMBERSHIP_LITERAL,
    );
    let (validation, counters) = validate_free_space_membership_block(artifact, scope);
    let worth_store_physical_integrity::FreeSpaceMembershipBlockIntegrityValidation::Intact(
        validated,
    ) = validation
    else {
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
    assert_eq!(
        entries[0].class(),
        worth_store_physical_format::RecordAllocationClass::InlinePage
    );
    assert_eq!(entries[0].owner(), 7);
    assert_eq!(
        entries[1].class(),
        worth_store_physical_format::RecordAllocationClass::ExtentArena
    );
    assert_eq!(entries[1].owner(), 5);
    let arena = entries[1].arena_free_range().unwrap();
    assert_eq!(arena.offset(), 100);
    assert_eq!(arena.length(), 1);
    assert_eq!(entries[1].generation(), 1);
    assert!(validated.children().is_none());
    assert!(validated.matches_input(artifact));
    let identical_copy = MEMBERSHIP_LITERAL.to_vec();
    assert!(!validated.matches_input(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &identical_copy
        )
    ));
    assert_intact_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceMembershipBlock,
        MEMBERSHIP_LITERAL.len() as u64,
    );
    assert!(validated.into_validation_record().matches_scope(scope));
}
