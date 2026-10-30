#![forbid(unsafe_code)]

mod box2;
pub use box2::{BoundsDenial, Box2, Box2Coordinates};
mod box_tree;
pub use box_tree::{build_box_tree, query_box_tree, BoxIndexDenial, BoxNode, BoxNodeLinks};
mod prefix_order;
mod prefix_tree;
pub use prefix_order::{compare_prefix_keys, prefix_order, PrefixOrder};
pub use prefix_tree::{
    build_prefix_tree, query_prefix_tree, PrefixBuildStorage, PrefixIndexDenial, PrefixNode,
    PrefixQueryStorage,
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
