//! Full-key ordering mechanics only; no allocation or Native loan is fabricated.
use super::*;
use crate::identity::data::{EntityId, PartitionId};
fn entity(partition: u32, slot: u64, generation: u32) -> RecordRef {
    RecordRef::Entity(EntityId::new(PartitionId(partition), slot, generation))
}
fn relation(partition: u32, slot: u64, generation: u32) -> RecordRef {
    RecordRef::Relation(RelationId::new(PartitionId(partition), slot, generation))
}
#[test]
fn ordered_allocation_streams_allow_class_interleaving_and_keep_full_generation() {
    let records = [
        relation(0, 7, 1),
        entity(0, 2, 1),
        relation(0, 7, 2),
        entity(0, 2, 2),
        entity(1, 0, 1),
        relation(2, 0, 1),
    ];
    let mut visited = 0;
    assert_eq!(
        ordered_counts(&records, |width| {
            visited += width;
            Ok::<_, ()>(())
        }),
        Ok(Some((3, 3)))
    );
    assert_eq!(visited, records.len() * size_of::<RecordRef>());
    assert_eq!(
        ordered_counts(std::iter::empty(), |_| Ok::<_, ()>(())),
        Ok(Some((0, 0)))
    );
}
#[test]
fn non_strict_allocations_require_the_general_collector() {
    for records in [
        [entity(0, 1, 1), entity(0, 1, 1)],
        [entity(1, 0, 1), entity(0, 99, 1)],
        [entity(0, 1, 2), entity(0, 1, 1)],
        [relation(0, 3, 1), relation(0, 2, 1)],
        [relation(1, 0, 1), relation(0, 99, 1)],
    ] {
        assert_eq!(
            ordered_counts(&records, |_| Ok::<_, ()>(())),
            Ok(None),
            "{records:?}"
        );
    }
}
#[test]
fn an_original_visit_stop_never_becomes_an_ordering_fallback() {
    let records = [entity(0, 2, 1), entity(0, 1, 1)];
    let mut visits = 0;
    let result = ordered_counts(&records, |_| {
        visits += 1;
        if visits == 2 {
            Err("original stop")
        } else {
            Ok(())
        }
    });
    assert_eq!(result, Err("original stop"));
    assert_eq!(visits, 2);
}
