//! A node may name more children than the source root still routes: earlier
//! batches of the same release dropped the rest.

use worth_store_physical_format::{
    BlobTreeEntryV1, BlobTreeNodeKind, BlobTreeNodeV1, BlobTreeOccurrenceV1,
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, ExtentArenaId, ExtentArenaRange,
    PersistedRecordIdentity, PhysicalExtentId, PhysicalGeneration, PhysicalGenerationAuthority,
    SelectedRecordContentClass, SelectedRecordRouteMetadata,
};

use super::super::closure_evidence::ReleasedClosureEvidence;
use super::{push_children, ExpectedEdge};

const NODE: u64 = 1;
const CHILDREN: std::ops::Range<u64> = 10..18;
const CHUNK: u64 = 64 << 10;

fn record(ordinal: u64) -> PersistedRecordIdentity {
    PersistedRecordIdentity::new([4; 16], ordinal).unwrap()
}

fn route(ordinal: u64) -> CurrentPhysicalRecordPlacement {
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .record_extent_cell(PhysicalExtentId::from_raw(ordinal).unwrap())
        .with_extent_generation(PhysicalGeneration::from_raw(1).unwrap());
    let range = ExtentArenaRange::new(ExtentArenaId::new(ordinal).unwrap(), 0, 64).unwrap();
    let metadata =
        SelectedRecordRouteMetadata::primary(SelectedRecordContentClass::Opaque).unwrap();
    CurrentPhysicalRecordPlacement::Extent(
        DurableExtentRecordPlacement::new_selected(record(ordinal), cell, 100, range, metadata)
            .unwrap(),
    )
}

/// One leaf node naming eight chunks.
fn node() -> BlobTreeNodeV1 {
    let occurrence =
        BlobTreeOccurrenceV1::new([1; 16], [2; 16], BlobTreeNodeKind::Leaf, 0, 0).unwrap();
    let entries = CHILDREN
        .map(|ordinal| BlobTreeEntryV1::new([ordinal as u8; 32], record(ordinal), CHUNK).unwrap())
        .collect();
    BlobTreeNodeV1::new(occurrence, entries).unwrap()
}

/// The node and every child it names, all still routed.
fn whole_routes() -> Vec<CurrentPhysicalRecordPlacement> {
    std::iter::once(NODE).chain(CHILDREN).map(route).collect()
}

fn push_whole(pending: &mut Vec<ExpectedEdge>, routes: &[CurrentPhysicalRecordPlacement]) -> bool {
    push_children(
        pending,
        &node(),
        record(NODE),
        0,
        routes,
        ReleasedClosureEvidence::FirstPublication,
    )
}

/// The ordinals queued when the source root routes the node and `routed`,
/// and `dropped` is what the retained history of the release dropped.
fn queued(routed: &[u64], dropped: &[u64]) -> Option<Vec<u64>> {
    let mut routes = vec![route(NODE)];
    routes.extend(routed.iter().copied().map(route));
    let dropped: Vec<_> = dropped.iter().copied().map(record).collect();
    let mut pending = Vec::new();
    push_children(
        &mut pending,
        &node(),
        record(NODE),
        0,
        &routes,
        ReleasedClosureEvidence::RetainedHistory(&dropped),
    )
    .then(|| pending.iter().map(|edge| edge.record.ordinal()).collect())
}

#[test]
fn a_node_with_more_children_than_routes_queues_only_the_children_still_routed() {
    assert_eq!(
        queued(&[16, 17], &[10, 11, 12, 13, 14, 15]),
        Some(vec![16, 17])
    );
    assert_eq!(queued(&[], &[10, 11, 12, 13, 14, 15, 16, 17]), Some(vec![]));
}

#[test]
fn a_child_neither_routed_nor_dropped_by_the_release_denies_the_closure() {
    assert_eq!(queued(&[16, 17], &[10, 11, 12, 13, 14]), None);
    assert_eq!(queued(&[17], &[]), None);
}

#[test]
fn every_routed_child_is_queued_at_its_own_offset() {
    let routes = whole_routes();
    let mut pending = Vec::new();
    assert!(push_whole(&mut pending, &routes));
    let observed: Vec<_> = pending
        .iter()
        .map(|edge| (edge.record.ordinal(), edge.entry_index, edge.start))
        .collect();
    let expected: Vec<_> = CHILDREN
        .enumerate()
        .map(|(index, ordinal)| (ordinal, index as u16, index as u64 * CHUNK))
        .collect();
    assert_eq!(observed, expected);
}

/// A tree queues each routed record once. A second parent naming the same
/// children is not a tree, and the queue stops at the size of the routes.
#[test]
fn children_named_again_by_another_parent_never_queue_past_the_routes() {
    let routes = whole_routes();
    let mut pending = Vec::new();
    assert!(push_whole(&mut pending, &routes));
    assert_eq!(pending.len(), routes.len() - 1);
    assert!(!push_whole(&mut pending, &routes));
    assert_eq!(pending.len(), routes.len());
}
