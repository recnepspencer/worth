//! The head-only successor oracle's own frontier rule.

use worth_store_physical_format::{
    DurablePhysicalRootManifest, ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadKeyV1,
};

use super::head_frontier_is_exactly_one_step as one_step;

/// A root whose head tree, if any, is rooted at `(generation, block)`.
fn root(generation: u64, head: Option<(u64, u64)>, frontier: u64) -> DurablePhysicalRootManifest {
    let key = ReleaseCustodyHeadKeyV1::new([1; 16], 1).unwrap();
    let head = head.map(|(generation, block)| {
        ReleaseCustodyHeadBlockReferenceV1::new(generation, block, 0, key, key, [1; 32]).unwrap()
    });
    DurablePhysicalRootManifest::builder(generation, 9, 2, 1)
        .release_custody_head_root(head)
        .next_release_custody_head_block(frontier)
        .admit()
        .unwrap()
}

#[test]
fn a_remaining_head_root_is_the_last_block_its_own_step_allocated() {
    let prior = root(5, Some((5, 3)), 4);
    assert!(one_step(&root(6, Some((6, 4)), 5), &prior), "one node");
    assert!(
        one_step(&root(6, Some((6, 6)), 7), &prior),
        "a three-node path"
    );
    assert!(
        !one_step(&root(6, Some((6, 4)), 6), &prior),
        "a block was allocated past the root"
    );
    assert!(
        !one_step(&root(6, Some((6, 3)), 4), &prior),
        "the root reuses a block below the source frontier"
    );
    assert!(
        !one_step(&root(6, Some((5, 4)), 5), &prior),
        "the root is stamped with another generation"
    );
}

#[test]
fn an_emptied_head_tree_allocates_no_block() {
    let prior = root(5, Some((5, 3)), 4);
    assert!(one_step(&root(6, None, 4), &prior));
    assert!(!one_step(&root(6, None, 5), &prior));
}
