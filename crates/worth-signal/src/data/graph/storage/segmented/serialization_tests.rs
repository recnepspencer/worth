use std::cell::Cell;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use super::{FlatSegments, Segment, SegmentedStorage, SegmentedStore};
use crate::data::graph::storage::handles::DependencySetId;

thread_local! {
    static ITEM_CLONES: Cell<usize> = const { Cell::new(0) };
}

#[derive(Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
struct Counted(u64);

#[derive(Serialize)]
struct ExpectedWire<'a> {
    items: &'a [Counted],
    segments: Vec<ExpectedSegment>,
}

#[derive(Serialize)]
struct ExpectedSegment {
    start: u32,
    len: u32,
}

impl Clone for Counted {
    fn clone(&self) -> Self {
        ITEM_CLONES.set(ITEM_CLONES.get() + 1);
        Self(self.0)
    }
}

fn literal_overlap_store() -> SegmentedStore<Counted, DependencySetId> {
    SegmentedStore {
        storage: SegmentedStorage::Exclusive(FlatSegments {
            items: (10..=16).map(Counted).collect(),
            segments: vec![
                Segment { start: 2, len: 2 },
                Segment { start: 6, len: 0 },
                Segment { start: 1, len: 3 },
            ],
        }),
        interner: crate::data::persistent_hash_map::PersistentHashMap::new(),
        id: PhantomData,
    }
}

fn expected_base_segments() -> Vec<ExpectedSegment> {
    vec![
        ExpectedSegment { start: 2, len: 2 },
        ExpectedSegment { start: 6, len: 0 },
        ExpectedSegment { start: 1, len: 3 },
    ]
}

fn expected_destination_segments() -> Vec<ExpectedSegment> {
    let mut segments = expected_base_segments();
    segments.extend([
        ExpectedSegment { start: 7, len: 2 },
        ExpectedSegment { start: 9, len: 1 },
    ]);
    segments
}

fn assert_restored_layout(actual: &[u8], expected_items: Vec<Counted>) {
    let restored: SegmentedStore<Counted, DependencySetId> =
        serde_json::from_slice(actual).expect("layout-preserving wire restores");
    let SegmentedStorage::Exclusive(restored) = restored.storage else {
        panic!("wire restoration must recover flat ordinary storage");
    };
    assert_eq!(restored.items, expected_items);
    assert_eq!(
        restored.segments,
        vec![
            Segment { start: 2, len: 2 },
            Segment { start: 6, len: 0 },
            Segment { start: 1, len: 3 },
            Segment { start: 7, len: 2 },
            Segment { start: 9, len: 1 },
        ]
    );
}

#[test]
fn exclusive_serialization_borrows_flat_truth_without_cloning_items() {
    for segment_count in [64_u64, 4_096, 65_536] {
        let mut store = SegmentedStore::<Counted, DependencySetId>::default();
        for value in 0..segment_count {
            store.insert_from_slice(&[Counted(value)]);
        }
        let SegmentedStorage::Exclusive(flat) = &store.storage else {
            panic!("ordinary store must remain flat");
        };
        assert_eq!(flat.items.len(), segment_count as usize);
        let expected_items = (0..segment_count).map(Counted).collect::<Vec<_>>();
        ITEM_CLONES.set(0);
        let expected = serde_json::to_vec(&ExpectedWire {
            items: &expected_items,
            segments: (0..segment_count)
                .map(|start| ExpectedSegment {
                    start: start as u32,
                    len: 1,
                })
                .collect(),
        })
        .expect("borrowed native wire serializes");
        let actual = serde_json::to_vec(&store).expect("ordinary segmented store serializes");

        assert_eq!(actual, expected, "scale {segment_count} wire changed");
        assert_eq!(
            ITEM_CLONES.get(),
            0,
            "scale {segment_count} serialization cloned stored items"
        );
    }
}

#[test]
fn fork_shared_serialization_preserves_current_destination_truth_without_cloning() {
    let mut source = SegmentedStore::<Counted, DependencySetId>::default();
    let inherited = source.insert_from_slice(&[Counted(1), Counted(2)]);
    let mut destination = source.fork_persistent();
    assert!(source.shares_storage_with(&destination));
    let appended = destination.insert_from_slice(&[Counted(3)]);

    ITEM_CLONES.set(0);
    let wire = serde_json::to_string(&destination).expect("shared destination serializes");
    assert_eq!(ITEM_CLONES.get(), 0, "shared serialization cloned items");
    let restored: SegmentedStore<Counted, DependencySetId> =
        serde_json::from_str(&wire).expect("shared destination wire restores");
    assert_eq!(restored.get(inherited), &[Counted(1), Counted(2)]);
    assert_eq!(restored.get(appended), &[Counted(3)]);
    assert_eq!(source.get(inherited), &[Counted(1), Counted(2)]);
    assert_eq!(source.live_segment_count(), 1);
}

#[test]
fn fork_wire_preserves_unused_items_and_original_segment_offsets() {
    let mut source = literal_overlap_store();
    let base_items = (10..=16).map(Counted).collect::<Vec<_>>();
    let expected_base = serde_json::to_vec(&ExpectedWire {
        items: &base_items,
        segments: expected_base_segments(),
    })
    .expect("literal base oracle serializes");
    assert_eq!(serde_json::to_vec(&source).unwrap(), expected_base);

    let mut destination = source.fork_persistent();
    assert!(source.shares_storage_with(&destination));
    assert_eq!(
        serde_json::to_vec(&source).unwrap(),
        expected_base,
        "converting the source to shared storage changed its wire"
    );
    destination.insert_from_slice(&[Counted(20), Counted(21)]);
    let inherited_empty = destination.insert_from_slice(&[]);
    assert_eq!(inherited_empty, DependencySetId::EMPTY);
    destination.insert_from_slice(&[Counted(30)]);
    let sibling = destination.clone();
    let expected_items = [10, 11, 12, 13, 14, 15, 16, 20, 21, 30]
        .into_iter()
        .map(Counted)
        .collect::<Vec<_>>();
    let expected_destination = serde_json::to_vec(&ExpectedWire {
        items: &expected_items,
        segments: expected_destination_segments(),
    })
    .expect("literal destination oracle serializes");
    ITEM_CLONES.set(0);
    let actual = serde_json::to_vec(&destination).unwrap();
    assert_eq!(actual, expected_destination);
    assert_eq!(serde_json::to_vec(&sibling).unwrap(), expected_destination);
    assert_eq!(ITEM_CLONES.get(), 0, "shared wire projection cloned items");

    assert_restored_layout(&actual, expected_items);
}
