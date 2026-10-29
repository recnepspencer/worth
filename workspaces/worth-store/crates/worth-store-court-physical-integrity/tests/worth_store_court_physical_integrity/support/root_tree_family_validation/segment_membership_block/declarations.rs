pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::{
    maximum_segment_manifest_pages, PhysicalPageId, PhysicalSegmentId,
    PhysicalSegmentMembershipBlock, SegmentManifestBlockReference, SegmentPageKey,
};
pub(crate) use worth_store_physical_integrity::{
    PhysicalArtifactScope, PhysicalBlastRadius, PhysicalByteRange, PhysicalDamageCause,
    PhysicalFormatField, PhysicalIntegrityRejection, PhysicalIntegrityValidationDigest,
    PhysicalIntegrityValidationMechanism, PhysicalIntegrityVersionAxis, UntrustedPhysicalArtifact,
};

pub(crate) use crate::literal_vectors::root_tree_family_validation::{
    SEGMENT_MEMBERSHIP_LEAF_COMPLETE_CRC32C, SEGMENT_MEMBERSHIP_LEAF_LITERAL,
};
pub(crate) use crate::support::root_tree_family_validation::{
    assert_damage, assert_rejected_counters, assert_segment_checksum_mutation_contract,
    assert_segment_mutation_contract, field_range, format, independent_crc32c,
    reseal_durable_frame, segment_branch, segment_leaf, segment_rejection as validate_rejection,
    segment_scope, segment_scope_with_reference, store, validate_segment_intact as validate_intact,
    SEGMENT_BLOCK_OFFSET,
};

pub(crate) fn segment_leaf_with(
    tree: u64,
    generation: u64,
    block: u64,
) -> PhysicalSegmentMembershipBlock {
    let baseline = segment_leaf();
    worth_store_physical_format::PhysicalSegmentMembershipBlock::leaf(
        tree,
        generation,
        block,
        baseline.entries().unwrap().to_vec(),
        8,
    )
    .unwrap()
}

pub(crate) fn assert_segment_damage(
    bytes: &[u8],
    scope: PhysicalArtifactScope,
    cause: PhysicalDamageCause,
    range: PhysicalByteRange,
    field: Option<PhysicalFormatField>,
    blast_radius: PhysicalBlastRadius,
) {
    let (rejection, counters) = validate_rejection(bytes, scope);
    assert_damage(rejection, scope, cause, range, field, blast_radius);
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::SegmentMembership,
        bytes.len() as u64,
        cause,
    );
}
