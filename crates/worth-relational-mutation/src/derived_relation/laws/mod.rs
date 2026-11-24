//! Graph laws over actual canonical endpoint pairs, independent of caching.
pub mod evaluate;
pub mod evidence;
pub use crate::contracts::derived_relation::laws::meaning::{
    DerivedRelationCardinality, DerivedRelationGraphLaws, DerivedRelationMinimumReachability,
};
pub use crate::contracts::derived_relation::laws::violation::{
    DerivedRelationLaw, DerivedRelationLawViolation,
};
