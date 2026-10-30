#[allow(unused_imports)]
use crate::free_space_validation::header::*;

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
    let artifact = worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
        HEADER_LITERAL,
    );
    let (validation, counters) = validate_free_space_header(artifact, scope);
    let worth_store_physical_integrity::FreeSpaceHeaderIntegrityValidation::Intact(validated) =
        validation
    else {
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
    assert!(!validated.matches_input(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &identical_copy
        )
    ));
    assert_intact_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::FreeSpaceHeader,
        HEADER_LITERAL.len() as u64,
    );
    assert!(validated.into_validation_record().matches_scope(scope));
}
