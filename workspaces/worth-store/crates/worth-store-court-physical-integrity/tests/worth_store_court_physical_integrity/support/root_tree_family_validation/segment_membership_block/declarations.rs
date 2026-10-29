pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::maximum_segment_manifest_pages, worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalSegmentId, worth_store_physical_format::PhysicalSegmentMembershipBlock, worth_store_physical_format::SegmentManifestBlockReference, worth_store_physical_format::SegmentPageKey};
pub(crate) use {worth_store_physical_integrity::PhysicalArtifactScope, worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalByteRange, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField, worth_store_physical_integrity::PhysicalIntegrityRejection, worth_store_physical_integrity::PhysicalIntegrityValidationDigest, worth_store_physical_integrity::PhysicalIntegrityValidationMechanism, worth_store_physical_integrity::PhysicalIntegrityVersionAxis, worth_store_physical_integrity::UntrustedPhysicalArtifact};

pub(crate) use {crate::literal_vectors::root_tree_family_validation::SEGMENT_MEMBERSHIP_LEAF_COMPLETE_CRC32C, crate::literal_vectors::root_tree_family_validation::SEGMENT_MEMBERSHIP_LEAF_LITERAL};
pub(crate) use {crate::support::root_tree_family_validation::assert_damage, crate::support::root_tree_family_validation::assert_rejected_counters, crate::support::root_tree_family_validation::assert_segment_checksum_mutation_contract, crate::support::root_tree_family_validation::assert_segment_mutation_contract, crate::support::root_tree_family_validation::field_range, crate::support::root_tree_family_validation::format, crate::support::root_tree_family_validation::independent_crc32c, crate::support::root_tree_family_validation::reseal_durable_frame, crate::support::root_tree_family_validation::segment_branch, crate::support::root_tree_family_validation::segment_leaf, crate::support::root_tree_family_validation::segment_rejection as validate_rejection, crate::support::root_tree_family_validation::segment_scope, crate::support::root_tree_family_validation::segment_scope_with_reference, crate::support::root_tree_family_validation::store, crate::support::root_tree_family_validation::validate_segment_intact as validate_intact, crate::support::root_tree_family_validation::SEGMENT_BLOCK_OFFSET};

pub(crate) fn segment_leaf_with(tree: u64, generation: u64, block: u64) -> PhysicalSegmentMembershipBlock {
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