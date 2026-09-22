use crate::identity::data::{KindId, PartitionId, VersionId};
use crate::storage::substrate::{EntityArena, EntityExtra, PinClass, SlotInit};

fn arena(slots: impl IntoIterator<Item = usize>) -> EntityArena {
    let mut arena = EntityArena::with_capacity(0);
    for slot in slots {
        arena
            .write_reserved_slot(
                SlotInit {
                    partition_id: PartitionId(1),
                    kind_id: KindId(1),
                    version_id: VersionId(1),
                    extra: EntityExtra::default(),
                },
                slot,
                1,
            )
            .unwrap();
    }
    arena
}

#[test]
fn preserving_unpinned_rows_never_materializes_zero_counters() {
    let current = arena(0..10_000);
    let mut changed = current.clone();
    changed.preserve_runtime_pins_from(&current);
    for pins in [
        &changed.branch_pins,
        &changed.replay_pins,
        &changed.snapshot_pins,
    ] {
        assert_eq!(pins.allocation_bytes(), 0);
        assert_eq!(pins.get(9999), Some(&0));
    }
    changed.reset_slot(9999);
    assert_eq!(changed.branch_pins.allocation_bytes(), 0);
    assert_eq!(changed.replay_pins.allocation_bytes(), 0);
    assert_eq!(changed.snapshot_pins.allocation_bytes(), 0);
}

#[test]
fn pins_remap_by_logical_slot_and_releasing_last_pin_removes_storage() {
    let mut current = arena([50_000, 90_000, 20]);
    current
        .adjust_named_pin(50_000, PinClass::Replay, 2)
        .unwrap();
    current.adjust_named_pin(20, PinClass::Branch, 1).unwrap();
    let mut reordered = arena([20, 90_000, 50_000]);
    reordered.preserve_runtime_pins_from(&current);
    assert_eq!(reordered.replay_pin_count(50_000), Some(2));
    assert_eq!(reordered.branch_pin_count(20), Some(1));
    assert_eq!(reordered.branch_pin_count(90_000), Some(0));
    reordered
        .adjust_named_pin(50_000, PinClass::Replay, -2)
        .unwrap();
    reordered
        .adjust_named_pin(20, PinClass::Branch, -1)
        .unwrap();
    assert_eq!(reordered.replay_pins.allocation_bytes(), 0);
    assert_eq!(reordered.branch_pins.allocation_bytes(), 0);
    assert_eq!(current.replay_pin_count(50_000), Some(2));
    assert_eq!(current.branch_pin_count(20), Some(1));
}
