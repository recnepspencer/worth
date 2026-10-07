use super::*;
use worth_store_physical_format::{
    DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange, ManifestBlockReference,
    PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
};

fn id(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([7; 16], ordinal).unwrap()
}

fn route(ordinal: u64) -> CurrentPhysicalRecordPlacement {
    let extent = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(ordinal).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::legacy_unknown(
            id(ordinal),
            extent,
            32,
            ExtentArenaRange::new(ExtentArenaId::new(1).unwrap(), (ordinal - 1) * 4096, 4096)
                .unwrap(),
        )
        .unwrap(),
    )
}

#[test]
fn leaf_removal_preserves_other_route_and_counts_one_descriptor_insert() {
    let original = vec![route(1), route(2)];
    let updates = BTreeMap::from([(id(3), route(3))]);
    let drops = BTreeSet::from([id(2)]);
    let merged = merge_leaf(original, &updates, &drops);
    assert_eq!(merged.entries, vec![route(1), route(3)]);
    assert_eq!((merged.inserted, merged.removed), (1, 1));
    assert_eq!(require_complete_drop(merged.removed, drops.len()), Ok(()));
}

#[test]
fn absent_drop_is_denied_and_complete_leaf_removal_has_empty_result() {
    let missing = BTreeSet::from([id(9)]);
    let unchanged = merge_leaf(vec![route(1)], &BTreeMap::new(), &missing);
    assert_eq!(unchanged.entries, vec![route(1)]);
    assert_eq!(unchanged.removed, 0);
    assert_eq!(
        require_complete_drop(unchanged.removed, missing.len()),
        Err(ManifestLookupFailure::Damaged),
    );

    let all = BTreeSet::from([id(1), id(2)]);
    let empty = merge_leaf(vec![route(1), route(2)], &BTreeMap::new(), &all);
    assert!(empty.entries.is_empty());
    assert_eq!(empty.removed, 2);
    assert_eq!(require_complete_drop(empty.removed, all.len()), Ok(()));
}

#[test]
fn multileaf_drop_targets_only_its_child_and_preserves_untouched_child() {
    let children = [
        ManifestBlockReference::new(1, 1, 0, 1, id(1), id(2)).unwrap(),
        ManifestBlockReference::new(1, 2, 0, 1, id(3), id(4)).unwrap(),
    ];
    let drops = BTreeSet::from([id(2)]);
    let mut discovery = super::super::super::ManifestDiscoveryCounterSnapshot::default();
    let assigned = super::super::assignment::assign_updates(
        &children,
        &BTreeMap::new(),
        &drops,
        &mut discovery,
    );
    assert_eq!(assigned[0].1, drops);
    assert!(assigned[1].0.is_empty() && assigned[1].1.is_empty());
    let left = merge_leaf(vec![route(1), route(2)], &assigned[0].0, &assigned[0].1);
    assert_eq!(left.entries, vec![route(1)]);
    // rewrite_children carries an unassigned authenticated child reference
    // unchanged rather than rewriting or dropping its routes.
    assert_eq!(children[1].first(), id(3));
    assert_eq!(children[1].last(), id(4));
}
