//! Fixed slot partitions cover every slot once; directory admission refuses
//! gaps, overlaps, missing tails and binding/entry mismatches.
use super::super::chunk_directory::{ChunkCoverageRefusal, RegionChunkDirectory};
use super::super::slot_range::{fixed_partition, SlotDomain, SlotRange, SlotRangeError};
use super::member_store::{seal, version, MemberStore, SlotFixture, LAYOUT};

#[test]
fn fixed_partition_assigns_every_slot_to_exactly_one_chunk() {
    for total in [0, 1, 63, 64, 65, 1_000] {
        let ranges = fixed_partition(total, 64).collect::<Vec<_>>();
        for slot in 0..total {
            let owners = ranges.iter().filter(|range| range.contains(slot)).count();
            assert_eq!(owners, 1, "slot {slot} of {total}");
            assert_eq!(
                SlotRange::fixed_chunk_of(slot, total, 64).filter(|range| range.contains(slot)),
                ranges.iter().copied().find(|range| range.contains(slot))
            );
        }
        assert!(ranges
            .iter()
            .all(|range| range.end_slot_exclusive() <= total));
        assert!(ranges.iter().all(|range| range.is_fixed_chunk(total, 64)));
    }
    assert_eq!(
        SlotRange::new(5, 5),
        Err(SlotRangeError::Empty { start: 5, end: 5 })
    );
    let empty = rmp_serde::to_vec_named(&serde_json::json!({
        "start_slot_inclusive": 9, "end_slot_exclusive": 2
    }))
    .unwrap();
    assert!(
        rmp_serde::from_slice::<SlotRange>(&empty).is_err(),
        "decode refuses empty ranges"
    );
}

fn sealed_domain(
    store: &MemberStore,
    domain: SlotDomain,
    total: u64,
) -> Vec<(
    super::super::semantic_entry::ChunkSemanticEntry,
    super::super::locator_binding::ChunkLocatorBinding,
)> {
    fixed_partition(total, LAYOUT.chunk_slots)
        .map(|range| {
            let slot = SlotFixture {
                slot: range.start_slot_inclusive(),
                head: Some(version(1, range.start_slot_inclusive())),
                history: Vec::new(),
                adjacency: Vec::new(),
            };
            let sealed = seal(store, domain, total, range, &[slot]);
            (sealed.semantic, sealed.binding)
        })
        .collect()
}

#[test]
fn directory_admits_exact_coverage_and_refuses_every_partition_defect() {
    let store = MemberStore::default();
    let entities = sealed_domain(&store, SlotDomain::Entity, 200);
    let relations = sealed_domain(&store, SlotDomain::Relation, 70);
    let all = || {
        entities
            .iter()
            .chain(&relations)
            .cloned()
            .collect::<Vec<_>>()
    };
    let admitted = RegionChunkDirectory::admit(LAYOUT, 200, 70, all()).expect("exact coverage");
    assert_eq!(admitted.chunks().len(), 4 + 2);
    assert!(admitted.chunk_of(SlotDomain::Entity, 199).is_some());
    assert!(admitted.chunk_of(SlotDomain::Entity, 200).is_none());

    let mut gap = all();
    gap.remove(1);
    let mut overlap = all();
    overlap.insert(1, overlap[0].clone());
    let mut missing_tail = all();
    missing_tail.pop();
    for (defect, chunks) in [("gap", gap), ("overlap", overlap), ("tail", missing_tail)] {
        assert!(
            matches!(
                RegionChunkDirectory::admit(LAYOUT, 200, 70, chunks),
                Err(ChunkCoverageRefusal::PartitionMismatch { .. })
            ),
            "{defect}"
        );
    }
    assert!(
        matches!(
            RegionChunkDirectory::admit(LAYOUT, 201, 70, all()),
            Err(ChunkCoverageRefusal::PartitionMismatch { .. })
        ),
        "a directory of other slot bounds is not this region's"
    );
    let mut swapped = all();
    let foreign_binding = swapped[1].1;
    swapped[0].1 = foreign_binding;
    assert_eq!(
        RegionChunkDirectory::admit(LAYOUT, 200, 70, swapped),
        Err(ChunkCoverageRefusal::BindingMismatch {
            slot_range: foreign_binding.slot_range()
        })
    );
}
