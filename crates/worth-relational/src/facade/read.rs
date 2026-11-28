//! Exact-root read and traversal contracts.
pub use super::runtime::{
    DerivedRelationReadDenial, DerivedRelationReadShape, PinnedDerivedRelationDenial,
    PinnedDerivedRelationRead, PinnedDerivedRelationScope,
    PinnedStructuralWalk, PinnedStructuralWalkDenial, PinnedStructuralWalkScope,
    RelationalPinnedRead, RelationalPinnedRead as RelationalReadPort,
    VisibilityProjectionView,
};
pub use crate::visibility::materialization::read_records::reader::query_execution::pinned_walk::{
    BoundedPinnedStructuralWalk, PinnedStructuralWalkBudget, PinnedStructuralWalkLimit,
};
