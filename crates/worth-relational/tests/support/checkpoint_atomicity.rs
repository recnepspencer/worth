//! Exact native refusal atomicity, shared by native and Query owner courts.
use super::{Class, RecordRef, RelationalCheckpointInspection};
use std::collections::BTreeMap;

pub(super) fn assert_model_unchanged(
    before: RelationalCheckpointInspection<'_>,
    after: RelationalCheckpointInspection<'_>,
    reservations: impl Iterator<Item = RecordRef>,
) {
    let RelationalCheckpointInspection {
        model,
        frontiers,
        generations,
        live_records,
    } = before;
    let RelationalCheckpointInspection {
        model: next_model,
        frontiers: next_frontiers,
        generations: next_generations,
        live_records: next_live,
    } = after;
    assert_eq!(model, next_model);
    assert_eq!(live_records, next_live);
    let mut expected_frontiers: BTreeMap<_, _> = frontiers
        .into_iter()
        .map(|f| ((f.class, f.partition_id), f.next_slot))
        .collect();
    let mut expected_generations: BTreeMap<_, _> = generations
        .into_iter()
        .map(|g| ((g.class, g.partition_id, g.slot), g.generation))
        .collect();
    for record in reservations {
        let (class, partition, slot, generation) = match record {
            RecordRef::Entity(id) => (
                Class::Entity,
                id.partition_value(),
                id.local_slot_value(),
                id.generation_value(),
            ),
            RecordRef::Relation(id) => (
                Class::Relation,
                id.partition_value(),
                id.local_slot_value(),
                id.generation_value(),
            ),
        };
        let frontier = expected_frontiers.entry((class, partition)).or_insert(0);
        assert!(slot <= *frontier, "report skipped an unreserved slot");
        *frontier = (*frontier).max(slot.checked_add(1).unwrap());
        let high_water = expected_generations
            .entry((class, partition, slot))
            .or_insert(0);
        assert_eq!(generation, high_water.checked_add(1).unwrap());
        *high_water = generation;
    }
    assert_eq!(
        expected_frontiers,
        next_frontiers
            .into_iter()
            .map(|f| ((f.class, f.partition_id), f.next_slot))
            .collect()
    );
    assert_eq!(
        expected_generations,
        next_generations
            .into_iter()
            .map(|g| ((g.class, g.partition_id, g.slot), g.generation))
            .collect()
    );
}
