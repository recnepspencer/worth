#[allow(unused_imports)]
use crate::root_tree_family_validation::reference_scope_evidence::*;

#[test]
fn root_reference_level_and_last_bound_are_independently_bound() {
    let bytes = root_leaf().encode(crate::support::root_tree_family_validation::format());
    let checksum = independent_crc32c(&[&bytes]);
    let observed = root_leaf().reference(checksum);
    let wrong_level = worth_store_physical_format::ManifestBlockReference::new(
        observed.generation(),
        observed.block(),
        1,
        checksum,
        observed.first(),
        observed.last(),
    )
    .unwrap();
    let level_scope =
        root_scope_with_reference(store(7), wrong_level, bytes.len() as u64, ROOT_BLOCK_OFFSET);
    assert_root_mutation_contract(&bytes, level_scope);
    assert_root_damage(&bytes, level_scope, field_range(level_scope, 64, 2));

    let wrong_last =
        worth_store_physical_format::PersistedRecordIdentity::new([0xa2; 16], 5).unwrap();
    let wrong_range = worth_store_physical_format::ManifestBlockReference::new(
        observed.generation(),
        observed.block(),
        observed.level(),
        checksum,
        observed.first(),
        wrong_last,
    )
    .unwrap();
    let range_scope =
        root_scope_with_reference(store(7), wrong_range, bytes.len() as u64, ROOT_BLOCK_OFFSET);
    let (rejection, counters) = root_rejection(&bytes, range_scope);
    assert_damage(
        rejection,
        range_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        field_range(range_scope, 88, 24),
        Some(worth_store_physical_integrity::PhysicalFormatField::RecordIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::RootRoutingBlock,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
    );
}

#[test]
fn segment_reference_level_and_last_bound_are_independently_bound() {
    let bytes = segment_leaf().encode(crate::support::root_tree_family_validation::format());
    let checksum = independent_crc32c(&[&bytes]);
    let observed = segment_leaf().reference(checksum);
    let wrong_level = worth_store_physical_format::SegmentManifestBlockReference::new(
        observed.generation(),
        observed.block(),
        1,
        checksum,
        observed.first(),
        observed.last(),
    )
    .unwrap();
    let level_scope = segment_scope_with_reference(
        store(7),
        wrong_level,
        bytes.len() as u64,
        SEGMENT_BLOCK_OFFSET,
    );
    assert_segment_mutation_contract(&bytes, level_scope);
    assert_segment_damage(&bytes, level_scope, field_range(level_scope, 64, 2));

    let wrong_last = worth_store_physical_format::SegmentPageKey::new(
        worth_store_physical_format::PhysicalSegmentId::from_raw(13).unwrap(),
        worth_store_physical_format::PhysicalPageId::from_raw(18).unwrap(),
    );
    let wrong_range = worth_store_physical_format::SegmentManifestBlockReference::new(
        observed.generation(),
        observed.block(),
        observed.level(),
        checksum,
        observed.first(),
        wrong_last,
    )
    .unwrap();
    let range_scope = segment_scope_with_reference(
        store(7),
        wrong_range,
        bytes.len() as u64,
        SEGMENT_BLOCK_OFFSET,
    );
    let (rejection, counters) = segment_rejection(&bytes, range_scope);
    assert_damage(
        rejection,
        range_scope,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
        field_range(range_scope, 88, 16),
        Some(worth_store_physical_integrity::PhysicalFormatField::RecordIdentity),
        worth_store_physical_integrity::PhysicalBlastRadius::ReachableSubtree,
    );
    assert_rejected_counters(
        counters,
        worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily::SegmentMembership,
        bytes.len() as u64,
        worth_store_physical_integrity::PhysicalDamageCause::ChildReferenceMismatch,
    );
}
