pub(crate) use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
pub(crate) use worth_store_physical_format::{
    ManifestBlockReference, PersistedRecordIdentity, PhysicalPageId, PhysicalSegmentId,
    SegmentManifestBlockReference, SegmentPageKey,
};
pub(crate) use worth_store_physical_integrity::{
    PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField,
};

pub(crate) use crate::support::root_tree_family_validation::{
    assert_damage, assert_rejected_counters, assert_root_mutation_contract,
    assert_segment_mutation_contract, field_range, independent_crc32c, root_leaf, root_rejection,
    root_scope_with_reference, segment_leaf, segment_rejection, segment_scope_with_reference,
    store, ROOT_BLOCK_OFFSET, SEGMENT_BLOCK_OFFSET,
};

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
