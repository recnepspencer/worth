//! Whole-coordinate selection preserves an immutable original, including its owner.
use super::*;
use crate::storage::substrate::StorageAllocation;

// Plain selection DATA, incapable of minting custody or serving a prefix.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PrefixCoordinates {
    source_identity: u64,
    member: (u64, u64),
    dependencies: Vec<(u64, u64)>,
    partition: u64,
    schema: [u8; 32],
    row: usize,
    opening: Vec<(u64, u32)>,
}
fn coordinates(source: u64) -> PrefixCoordinates {
    PrefixCoordinates {
        source_identity: source,
        member: (source + 10, 31),
        dependencies: vec![(source + 20, 37)],
        partition: source + 30,
        schema: [source as u8; 32],
        row: source as usize + 40,
        opening: vec![(source + 50, 3), (source + 60, 4)],
    }
}

#[test]
fn adopting_a_different_head_carries_its_owned_historical_projection_with_the_record() {
    let lane = crate::storage::substrate::maintenance::ReconstructionLane::oracle();
    let mut destination_slots = RecordSlotDirectory::restore(&lane, vec![90, 13]).unwrap();
    let selected_slots = RecordSlotDirectory::restore(&lane, vec![22, 90, 13]).unwrap();
    let original: SharedColumn<StorageAllocation<Option<PrefixCoordinates>>> =
        vec![Some(coordinates(1)), Some(coordinates(2))].into();
    let selected: SharedColumn<StorageAllocation<Option<PrefixCoordinates>>> = vec![
        Some(coordinates(3)),
        Some(coordinates(4)),
        Some(coordinates(5)),
    ]
    .into();
    let original_ids = (
        original.retained_value(0).unwrap().id(),
        original.retained_value(1).unwrap().id(),
    );
    let mut destination = original.clone();
    for logical in [13, 22] {
        let plan = SlotAdoption::prepare(logical, &destination_slots, &selected_slots).unwrap();
        if plan.appends() {
            destination_slots.insert(logical as u64).unwrap();
        }
        let selected_backing = selected.retained_value(plan.selected_physical()).unwrap();
        plan.copy_selected_value(&mut destination, &selected);
        let adopted = destination
            .retained_value(plan.destination_physical())
            .unwrap();
        assert_eq!(adopted.id(), selected_backing.id());
        assert_eq!(&*adopted, &*selected_backing);
        assert_ne!(adopted.as_ref(), &original[1]);
    }
    assert_eq!(destination[0], Some(coordinates(1)));
    assert_eq!(destination.retained_value(0).unwrap().id(), original_ids.0);
    assert_eq!(original[0], Some(coordinates(1)));
    assert_eq!(original[1], Some(coordinates(2)));
    assert_eq!(original.retained_value(1).unwrap().id(), original_ids.1);
    assert!(SlotAdoption::prepare(99, &destination_slots, &selected_slots).is_none());
}
