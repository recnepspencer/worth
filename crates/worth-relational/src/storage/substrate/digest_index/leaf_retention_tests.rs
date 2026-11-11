//! A retained private Patricia leaf preserves its exact key and topology.
use super::*;
#[test]
fn escaped_patricia_leaf_keeps_original_key_after_tree_retirement() {
    let entries = [(0u128, [1u8; 32]), (1, [2; 32]), (2, [3; 32])];
    let tree = DigestIndex::from_sorted_entries(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &entries,
    );
    let original = tree.root.as_ref().unwrap();
    assert_eq!(original.nodes, 5);
    let leaf = original.children.as_ref().unwrap()[1].clone();
    assert!(leaf.children.is_none());
    drop(tree);
    assert_eq!(leaf.key, 2);
}
