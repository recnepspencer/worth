pub(crate) use {worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily};
pub(crate) use {worth_store_physical_format::ManifestBlockReference, worth_store_physical_format::PersistedRecordIdentity, worth_store_physical_format::PhysicalPageId, worth_store_physical_format::PhysicalSegmentId, worth_store_physical_format::SegmentManifestBlockReference, worth_store_physical_format::SegmentPageKey};
pub(crate) use {worth_store_physical_integrity::PhysicalBlastRadius, worth_store_physical_integrity::PhysicalDamageCause, worth_store_physical_integrity::PhysicalFormatField};

pub(crate) use {crate::support::root_tree_family_validation::assert_damage, crate::support::root_tree_family_validation::assert_rejected_counters, crate::support::root_tree_family_validation::assert_root_mutation_contract, crate::support::root_tree_family_validation::assert_segment_mutation_contract, crate::support::root_tree_family_validation::field_range, crate::support::root_tree_family_validation::independent_crc32c, crate::support::root_tree_family_validation::root_leaf, crate::support::root_tree_family_validation::root_rejection, crate::support::root_tree_family_validation::root_scope_with_reference, crate::support::root_tree_family_validation::segment_leaf, crate::support::root_tree_family_validation::segment_rejection, crate::support::root_tree_family_validation::segment_scope_with_reference, crate::support::root_tree_family_validation::store, crate::support::root_tree_family_validation::ROOT_BLOCK_OFFSET, crate::support::root_tree_family_validation::SEGMENT_BLOCK_OFFSET};

pub(crate) fn assert_root_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    range: worth_store_physical_integrity::PhysicalByteRange,
) {
    let (rejection, counters) = root_rejection(bytes, scope);
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        range,
        Some(worth_store_physical_integrity::PhysicalFormatField::ChildReference),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootRoutingBlock,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
    );
}

pub(crate) fn assert_segment_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    range: worth_store_physical_integrity::PhysicalByteRange,
) {
    let (rejection, counters) = segment_rejection(bytes, scope);
    assert_damage(
        rejection,
        scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        range,
        Some(worth_store_physical_integrity::PhysicalFormatField::ChildReference),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::SegmentMembership,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
    );
}