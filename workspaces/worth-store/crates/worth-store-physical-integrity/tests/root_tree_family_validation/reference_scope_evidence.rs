use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily;
use worth_store_physical_format::{
    ManifestBlockReference, PersistedRecordIdentity, PhysicalPageId, PhysicalSegmentId,
    SegmentManifestBlockReference, SegmentPageKey,
};
use worth_store_physical_integrity::{
    PhysicalBlastRadius, PhysicalDamageCause, PhysicalFormatField,
};

use super::support::{
    assert_damage, assert_rejected_counters, assert_root_mutation_contract,
    assert_segment_mutation_contract, field_range, independent_crc32c, root_leaf, root_rejection,
    root_scope_with_reference, segment_leaf, segment_rejection, segment_scope_with_reference,
    store, ROOT_BLOCK_OFFSET, SEGMENT_BLOCK_OFFSET,
};

#[test]
fn root_reference_level_and_last_bound_are_independently_bound() {
    let bytes = root_leaf().encode(super::support::format());
    let checksum = independent_crc32c(&[&bytes]);
    let observed = root_leaf().reference(checksum);
    let wrong_level = ManifestBlockReference::new(
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

    let wrong_last = PersistedRecordIdentity::new([0xa2; 16], 5).unwrap();
    let wrong_range = ManifestBlockReference::new(
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
        PhysicalDamageCause::ChildReferenceMismatch,
        field_range(range_scope, 88, 24),
        Some(PhysicalFormatField::RecordIdentity),
        PhysicalBlastRadius::ReachableSubtree,
    );
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::RootRoutingBlock,
        bytes.len() as u64,
        PhysicalDamageCause::ChildReferenceMismatch,
    );
}

#[test]
fn segment_reference_level_and_last_bound_are_independently_bound() {
    let bytes = segment_leaf().encode(super::support::format());
    let checksum = independent_crc32c(&[&bytes]);
    let observed = segment_leaf().reference(checksum);
    let wrong_level = SegmentManifestBlockReference::new(
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

    let wrong_last = SegmentPageKey::new(
        PhysicalSegmentId::from_raw(13).unwrap(),
        PhysicalPageId::from_raw(18).unwrap(),
    );
    let wrong_range = SegmentManifestBlockReference::new(
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
        PhysicalDamageCause::ChildReferenceMismatch,
        field_range(range_scope, 88, 16),
        Some(PhysicalFormatField::RecordIdentity),
        PhysicalBlastRadius::ReachableSubtree,
    );
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::SegmentMembership,
        bytes.len() as u64,
        PhysicalDamageCause::ChildReferenceMismatch,
    );
}

fn assert_root_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    range: worth_store_physical_integrity::PhysicalByteRange,
) {
    let (rejection, counters) = root_rejection(bytes, scope);
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        range,
        Some(PhysicalFormatField::ChildReference),
        PhysicalBlastRadius::ReachableSubtree,
    );
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::RootRoutingBlock,
        bytes.len() as u64,
        PhysicalDamageCause::ChildReferenceMismatch,
    );
}

fn assert_segment_damage(
    bytes: &[u8],
    scope: worth_store_physical_integrity::PhysicalArtifactScope,
    range: worth_store_physical_integrity::PhysicalByteRange,
) {
    let (rejection, counters) = segment_rejection(bytes, scope);
    assert_damage(
        rejection,
        scope,
        PhysicalDamageCause::ChildReferenceMismatch,
        range,
        Some(PhysicalFormatField::ChildReference),
        PhysicalBlastRadius::ReachableSubtree,
    );
    assert_rejected_counters(
        counters,
        PhysicalIntegrityArtifactFamily::SegmentMembership,
        bytes.len() as u64,
        PhysicalDamageCause::ChildReferenceMismatch,
    );
}
