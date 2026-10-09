//! Stable occupancy bounds preserve independent roots and checked capacity.
use super::{ordered_node_bytes, retained_forest_bytes, retained_map_bytes};

#[test]
fn retained_tree_bound_covers_sparse_root_and_minimum_children() {
    let node = ordered_node_bytes::<usize, ()>().unwrap();
    // A retained root may be empty or contain one key. At the first possible
    // split level, one root key plus two 31-key children needs three nodes.
    for (entries, maximum_nodes) in [(0, 1), (1, 1), (31, 1), (32, 2), (63, 3), (64, 3)] {
        assert_eq!(
            retained_map_bytes::<usize, ()>(entries),
            Some(node * maximum_nodes),
        );
    }
    // A minimal three-level tree has root 1, two internal nodes of 31 keys,
    // and 64 leaves of 31 keys: 2,047 entries and 67 nodes.
    assert_eq!(retained_map_bytes::<usize, ()>(2047), Some(node * 67));
}

#[test]
fn retained_forest_prices_singleton_and_empty_roots_separately() {
    let node = ordered_node_bytes::<usize, ()>().unwrap();
    let one_tree = retained_map_bytes::<usize, ()>(128).unwrap();
    let singletons = retained_forest_bytes::<usize, ()>(128, 128).unwrap();
    assert!(singletons >= 128 * node);
    assert!(singletons > 25 * one_tree);
    assert_eq!(retained_forest_bytes::<usize, ()>(0, 128), Some(128 * node));
    assert_eq!(retained_forest_bytes::<usize, ()>(0, 0), Some(0));
    assert_eq!(retained_forest_bytes::<usize, ()>(1, 0), None);
}

#[test]
fn retained_tree_forecasts_refuse_unrepresentable_capacity() {
    assert_eq!(
        retained_map_bytes::<[u8; 4096], [u8; 4096]>(usize::MAX),
        None
    );
    assert_eq!(
        retained_forest_bytes::<usize, ()>(usize::MAX, usize::MAX),
        None
    );
}
