//! Stable retained roots are distinct from transient insertion allocations.

use super::*;

#[test]
fn stable_node_quote_preserves_root_and_nonroot_occupancy_boundaries() {
    // Independent minimum occupancy: one key in the root, five in every
    // nonroot. These cardinal boundaries include singletons and the first
    // split's possible stable three-node image (one root, two children).
    let node = node_bytes::<Address, Posting>().unwrap();
    for (entries, upper_nodes) in [
        (0, 0),
        (1, 1),
        (5, 1),
        (6, 2),
        (10, 2),
        (11, 3),
        (12, 3),
        (70, 14),
        (71, 15),
        (72, 15),
        (430, 86),
        (431, 87),
        (432, 87),
    ] {
        assert_eq!(
            tree_retained_bytes::<Address, Posting>(entries),
            Some(upper_nodes * node)
        );
    }
    assert_eq!(tree_retained_bytes::<Address, Posting>(usize::MAX), None);
    // A singleton's actual stable image requires one node. Its insertion
    // still admits independent split/scratch headroom, rather than retaining
    // a second fictitious node for the lifetime of every source bucket.
    assert!(tree_insert_bytes::<Address, Posting>(0).unwrap() > node as u64);
}
