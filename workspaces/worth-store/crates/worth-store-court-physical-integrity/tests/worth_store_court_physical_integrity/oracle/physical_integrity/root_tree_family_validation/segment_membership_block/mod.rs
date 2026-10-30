#[allow(unused_imports)]
use crate::root_tree_family_validation::segment_membership_block::*;

#[test]
fn membership_literal_vector_has_independent_complete_child_crc_and_validates() {
    assert_eq!(
        segment_leaf().encode(format()),
        SEGMENT_MEMBERSHIP_LEAF_LITERAL
    );
    assert_eq!(
        independent_crc32c(&[&SEGMENT_MEMBERSHIP_LEAF_LITERAL]),
        SEGMENT_MEMBERSHIP_LEAF_COMPLETE_CRC32C
    );
    assert_eq!(
        u32::from_le_bytes(SEGMENT_MEMBERSHIP_LEAF_LITERAL[44..48].try_into().unwrap()),
        independent_crc32c(&[
            &SEGMENT_MEMBERSHIP_LEAF_LITERAL[..44],
            &SEGMENT_MEMBERSHIP_LEAF_LITERAL[48..],
        ])
    );
    let key = worth_store_physical_format::SegmentPageKey::new(
        worth_store_physical_format::PhysicalSegmentId::from_raw(13).unwrap(),
        worth_store_physical_format::PhysicalPageId::from_raw(17).unwrap(),
    );
    let reference = worth_store_physical_format::SegmentManifestBlockReference::new(
        11,
        5,
        0,
        SEGMENT_MEMBERSHIP_LEAF_COMPLETE_CRC32C,
        key,
        key,
    )
    .unwrap();
    let scope = segment_scope_with_reference(
        store(7),
        reference,
        SEGMENT_MEMBERSHIP_LEAF_LITERAL.len() as u64,
        SEGMENT_BLOCK_OFFSET,
    );
    let validated = validate_intact(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &SEGMENT_MEMBERSHIP_LEAF_LITERAL,
        ),
        scope,
    );
    assert_eq!(validated.entries().unwrap().len(), 1);
    let record = validated.into_validation_record();
    assert_eq!(
        record.byte_range_digest(),
        worth_store_physical_integrity::PhysicalIntegrityValidationDigest::crc32c(
            SEGMENT_MEMBERSHIP_LEAF_COMPLETE_CRC32C
        )
    );
    assert_eq!(
        record.mechanism(),
        worth_store_physical_integrity::PhysicalIntegrityValidationMechanism::Crc32cV1
    );
    assert_eq!(record.mechanism().version(), 1);
}
