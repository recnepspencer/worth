use worth_foundational::PartitionIdentity;

use super::{Bisection, BisectionDenial, WeightedItem};
use crate::partition::PartitionItemId;

#[test]
fn exhausted_cut_path_denial_preserves_retained_tree_routes_and_items() {
    let mut bisection = Bisection::new(1, 0).unwrap();
    let first = WeightedItem {
        item: PartitionItemId(1),
        weight: 1,
    };
    bisection.upsert_item(first).unwrap();

    // Put a valid one-item leaf at the deepest representable cut path. Adding
    // another item must split it, but neither child has a u64 identity.
    let deepest = 1_u64 << 63;
    bisection.tree.as_mut().unwrap().path = deepest;
    bisection
        .routes
        .insert(first.item, PartitionIdentity::new(deepest));
    let before_leaves = bisection.leaves();
    let before_route = bisection.route(first.item);
    let second = WeightedItem {
        item: PartitionItemId(2),
        weight: 1,
    };
    assert_eq!(
        bisection.upsert_item(second),
        Err(BisectionDenial::PathExhausted)
    );
    assert_eq!(bisection.leaves(), before_leaves);
    assert_eq!(bisection.route(first.item), before_route);
    assert!(bisection.route(second.item).is_none());
    assert!(!bisection.items.contains_key(&second.item));
    assert!(!bisection.incident.contains_key(&second.item));
}
