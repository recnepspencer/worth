use super::*;
use worth_foundational::facade::{AspectValue, FieldKey, InternedString};

fn make_fields(field_count: usize, payload_len: usize) -> Vec<(FieldKey, AspectValue)> {
    (0..field_count)
        .map(|idx| {
            (
                FieldKey::new(format!("Field{idx:03}")).unwrap(),
                AspectValue::String(InternedString::Raw("a".repeat(payload_len))),
            )
        })
        .collect()
}

#[test]
fn entity_backfill_bytes_covers_struct_tree_backing_containers_and_payloads() {
    let small = make_fields(1, 8);
    let wide = make_fields(32, 8);
    let large = make_fields(1, 64 * 1024);

    let b_small =
        struct_attachment_retained_bytes(small.iter().map(|(k, v)| (Some(k), v))).unwrap();
    let b_wide = struct_attachment_retained_bytes(wide.iter().map(|(k, v)| (Some(k), v))).unwrap();
    let b_large =
        struct_attachment_retained_bytes(large.iter().map(|(k, v)| (Some(k), v))).unwrap();

    let exact_tree_32 = StructAspectValue::sorted_construction_storage_bytes(32).unwrap() as u64;
    assert!(b_wide >= exact_tree_32);
    assert!(b_wide > b_small * 10);
    assert!(b_large >= 64 * 1024);

    // First BTreeMap insertion covers a full B=6 leaf node (>= 11*(K+V)).
    let single_node = btree_map_retained_bytes::<AspectFieldLocator, AspectValue>(1).unwrap();
    let min_leaf = (11 * (size_of::<AspectFieldLocator>() + size_of::<AspectValue>())) as u64;
    assert!(single_node >= min_leaf);

    // Pre-sized Vec container accounting is exact to `count * size_of::<T>()`.
    assert_eq!(
        vec_backing_bytes::<&usize>(64).unwrap(),
        (64 * size_of::<&usize>()) as u64
    );
    assert_eq!(
        vec_backing_bytes::<WorkerIntentBatch>(4).unwrap(),
        (4 * size_of::<WorkerIntentBatch>()) as u64
    );

    let group_large = planned_locator_and_value_bytes(
        "TaskPriority",
        "Priority",
        &AspectValue::String(InternedString::Raw("x".repeat(32 * 1024))),
    )
    .unwrap();
    assert!(group_large >= 32 * 1024);

    let batch_bytes = batch_allocation_bytes(128, 16).unwrap();
    assert!(batch_bytes >= 128 + (16 * size_of::<MutationIntent>()) as u64);
}
