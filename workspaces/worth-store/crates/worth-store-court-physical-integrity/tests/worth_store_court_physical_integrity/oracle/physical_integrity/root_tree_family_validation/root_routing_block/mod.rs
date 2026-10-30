#[allow(unused_imports)]
use crate::root_tree_family_validation::root_routing_block::*;

#[test]
fn root_routing_literal_vector_has_independent_complete_child_crc_and_validates() {
    assert_eq!(root_leaf().encode(format()), ROOT_ROUTING_LEAF_LITERAL);
    assert_eq!(
        independent_crc32c(&[&ROOT_ROUTING_LEAF_LITERAL]),
        ROOT_ROUTING_LEAF_COMPLETE_CRC32C
    );
    assert_eq!(
        u32::from_le_bytes(ROOT_ROUTING_LEAF_LITERAL[44..48].try_into().unwrap()),
        independent_crc32c(&[
            &ROOT_ROUTING_LEAF_LITERAL[..44],
            &ROOT_ROUTING_LEAF_LITERAL[48..],
        ])
    );
    let record = worth_store_physical_format::PersistedRecordIdentity::new([0xa1; 16], 5).unwrap();
    let reference = worth_store_physical_format::ManifestBlockReference::new(
        11,
        3,
        0,
        ROOT_ROUTING_LEAF_COMPLETE_CRC32C,
        record,
        record,
    )
    .unwrap();
    let scope = root_scope_with_reference(
        store(7),
        reference,
        ROOT_ROUTING_LEAF_LITERAL.len() as u64,
        ROOT_BLOCK_OFFSET,
    );
    let validated = validate_intact(
        worth_store_physical_integrity::UntrustedPhysicalArtifact::from_bounded_bytes(
            &ROOT_ROUTING_LEAF_LITERAL,
        ),
        scope,
    );
    assert_eq!(validated.entries().unwrap().len(), 1);
    let record = validated.into_validation_record();
    assert_eq!(
        record.byte_range_digest(),
        worth_store_physical_integrity::PhysicalIntegrityValidationDigest::crc32c(
            ROOT_ROUTING_LEAF_COMPLETE_CRC32C
        )
    );
    assert_eq!(
        record.mechanism(),
        worth_store_physical_integrity::PhysicalIntegrityValidationMechanism::Crc32cV1
    );
    assert_eq!(record.mechanism().version(), 1);
}
