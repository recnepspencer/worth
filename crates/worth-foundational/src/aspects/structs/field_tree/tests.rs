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
