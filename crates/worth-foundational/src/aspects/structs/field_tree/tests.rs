use super::*;
use std::collections::BTreeMap;

fn check(link: &Link) -> (usize, usize) {
    let Some(node) = link else {
        return (0, 0);
    };
    let (left_height, left_len) = check(&node.left);
    let (right_height, right_len) = check(&node.right);
    assert!(
        left_height.abs_diff(right_height) <= 1,
        "AVL path remains balanced"
    );
    assert_eq!(node.height, 1 + left_height.max(right_height));
    assert_eq!(node.len, 1 + left_len + right_len);
    (node.height, node.len)
}
fn equal(tree: &StructFields, oracle: &BTreeMap<FieldKey, Arc<AspectValue>>) {
    assert_eq!(tree.len(), oracle.len());
    assert!(
        tree.iter().eq(oracle.iter()),
        "borrowed order equals independent sorted storage"
    );
    check(&tree.root);
}
#[test]
fn mixed_fields_keep_order_balance_and_immutable_predecessors() {
    let mut tree = StructFields::new();
    let mut oracle = BTreeMap::new();
    let mut seed = 71_u64;
    for step in 0..2048 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let key = FieldKey::new(format!("field-{:03}", (seed >> 32) % 193)).unwrap();
        let old = tree.clone();
        let old_oracle = oracle.clone();
        if step % 3 == 0 {
            tree.remove(&key);
            oracle.remove(&key);
        } else {
            let value = Arc::new(AspectValue::UInt64(step));
            tree.insert(key.clone(), value.clone());
            oracle.insert(key.clone(), value);
        }
        assert_eq!(tree.get(&key), oracle.get(&key));
        equal(&tree, &oracle);
        equal(&old, &old_oracle);
    }
}
#[test]
fn doubled_field_population_keeps_navigation_height_in_the_log_growth_class() {
    let mut heights = Vec::new();
    for count in [128, 256] {
        let mut tree = StructFields::new();
        for index in 0..count {
            tree.insert(
                FieldKey::new(format!("field-{index:04}")).unwrap(),
                Arc::new(AspectValue::UInt64(index)),
            );
        }
        heights.push(check(&tree.root).0);
    }
    assert!(heights[0] > 0);
    assert!(
        heights[1] * 2 <= heights[0] * 3,
        "field navigation grows beyond the log class: {heights:?}"
    );
}

fn collect_stamps(
    link: &Link,
    out: &mut BTreeMap<FieldKey, (Option<u64>, Option<u64>, Option<u64>)>,
) {
    let Some(node) = link else { return };
    collect_stamps(&node.left, out);
    out.insert(
        (*node.entry.key).clone(),
        (
            node.birth_seq,
            node.entry.key_birth_seq,
            node.entry.value_birth_seq,
        ),
    );
    collect_stamps(&node.right, out);
}

#[test]
fn rotations_and_replacements_preserve_independent_field_birth_sequences() {
    let initial_fields = vec![
        (FieldKey::new("b").unwrap(), AspectValue::UInt64(2)),
        (FieldKey::new("d").unwrap(), AspectValue::UInt64(4)),
        (FieldKey::new("f").unwrap(), AspectValue::UInt64(6)),
    ];
    let mut tree = StructFields::from_sorted_fields(initial_fields.into_iter());
    let watermark = crate::aspects::current_foundational_allocation_watermark().unwrap();
    let mut initial_stamps = BTreeMap::new();
    collect_stamps(&tree.root, &mut initial_stamps);
    for (node_seq, key_seq, val_seq) in initial_stamps.values() {
        assert!(node_seq.unwrap() < watermark);
        assert!(key_seq.unwrap() < watermark);
        assert!(val_seq.unwrap() < watermark);
    }

    // Insert fresh fields forcing path copies and AVL rotations of "b", "d", "f".
    for (name, v) in [("a", 1), ("c", 3), ("e", 5), ("g", 7), ("h", 8), ("i", 9)] {
        tree.insert_fresh(FieldKey::new(name).unwrap(), AspectValue::UInt64(v));
    }
    let mut rotated_stamps = BTreeMap::new();
    collect_stamps(&tree.root, &mut rotated_stamps);
    let mut saw_rotated_old_field = false;
    for (k, &(old_node, old_key, old_val)) in &initial_stamps {
        let (new_node, new_key, new_val) = rotated_stamps[k];
        if new_node != old_node {
            assert!(new_node.unwrap() >= watermark);
            saw_rotated_old_field = true;
        }
        assert_eq!(new_key, old_key, "rotation must not restamp reused FieldKey");
        assert_eq!(new_val, old_val, "rotation must not restamp reused AspectValue");
        assert!(new_key.unwrap() < watermark);
        assert!(new_val.unwrap() < watermark);
    }
    assert!(saw_rotated_old_field, "inserts must rotate at least one prior node");
    for name in ["a", "c", "e", "g", "h", "i"] {
        let k = FieldKey::new(name).unwrap();
        let (node_seq, key_seq, val_seq) = rotated_stamps[&k];
        assert!(node_seq.unwrap() >= watermark);
        assert!(key_seq.unwrap() >= watermark);
        assert!(val_seq.unwrap() >= watermark);
    }

    // Replace existing key "d" with an external Arc<AspectValue>: key keeps its old
    // pre-watermark birth_seq while the unknown-age external value receives None.
    let key_d = FieldKey::new("d").unwrap();
    tree.insert(key_d.clone(), Arc::new(AspectValue::UInt64(40)));
    let mut replaced_stamps = BTreeMap::new();
    collect_stamps(&tree.root, &mut replaced_stamps);
    assert_eq!(replaced_stamps[&key_d].1, initial_stamps[&key_d].1);
    assert_eq!(replaced_stamps[&key_d].2, None);

    // Remove "b" (two-child remove_min promotion of "c"): "c" keeps its exact stamps.
    let key_c = FieldKey::new("c").unwrap();
    let c_before = replaced_stamps[&key_c];
    tree.remove(&FieldKey::new("b").unwrap());
    let mut removed_stamps = BTreeMap::new();
    collect_stamps(&tree.root, &mut removed_stamps);
    assert_eq!(removed_stamps[&key_c].1, c_before.1);
    assert_eq!(removed_stamps[&key_c].2, c_before.2);
}
