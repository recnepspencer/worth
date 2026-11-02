//! Allocation custody for retained runtime data, separate from execution results.
pub(crate) mod ordered;
pub use ordered::{
    ExecutionOrderedMap, ExecutionOrderedMapCell, ExecutionOrderedMapDenial,
    ExecutionOrderedMapUniqueDenial,
};

mod ordered_tree;
pub use ordered_tree::{
    build_ordered_tree, build_sorted_ordered_tree, insert_ordered_tree, remove_ordered_tree,
    ExecutionOrderedTreeBounds, ExecutionOrderedTreeNode, ExecutionOrderedTreeStorage,
};
