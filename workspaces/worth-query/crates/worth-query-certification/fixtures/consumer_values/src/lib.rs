#![forbid(unsafe_code)]

mod box2;
mod segment_box;
pub use box2::{BoundsDenial, Box2, Box2Coordinates};
pub use segment_box::{endpoint_segment_intersects_box, segment_intersects_box, SegmentBoxDenial};
mod box_tree;
pub use box_tree::{
    build_box_tree, compare_box_bounds, insert_box_source, query_box_tree, remove_box_source,
    replace_box_source, BoxEditStorage, BoxIndexDenial, BoxNode, BoxNodeLinks,
};
mod prefix_order;
mod prefix_summary;
mod prefix_watch;
pub use prefix_summary::{join_prefix_bounds, prefix_bounds_may_match};
pub use prefix_watch::prefix_watch_bounds_may_match;
mod prefix_tree;
pub use prefix_order::{compare_prefix_keys, prefix_order, PrefixOrder};
pub use prefix_tree::{query_prefix_tree, PrefixIndexDenial, PrefixQueryStorage};
pub use worth_execution::{
    build_ordered_tree as build_prefix_tree, insert_ordered_tree as insert_prefix_source,
    remove_ordered_tree as remove_prefix_source, ExecutionOrderedTreeBounds as PrefixSourceBounds,
    ExecutionOrderedTreeNode as PrefixNode, ExecutionOrderedTreeStorage as PrefixBuildStorage,
};

mod planar_operation;
mod planar_replacement;
pub use planar_replacement::{
    PlanarReplacementDenial, PlanarVertexReplacement, PlanarVertexReplacementResult,
};

pub use planar_operation::{
    PlanarAdjustment, PlanarAdjustmentResult, PlanarCurrentOutputExpectation, PlanarDerivedOutput,
    PlanarMutationDenial, PlanarOperation, PlanarVertex,
};

mod positive_length;
pub use positive_length::PositiveLength;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PositiveCount(u64);

impl PositiveCount {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            None
        } else {
            Some(Self(value))
        }
    }

    pub const fn get(value: &Self) -> u64 {
        value.0
    }
}
