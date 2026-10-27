//! Graph laws over actual canonical endpoint pairs, independent of caching.
pub(crate) mod canonical;
pub(crate) mod evaluate;
mod evidence;
mod meaning;
mod violation;
pub use meaning::{
    DerivedRelationCardinality, DerivedRelationGraphLaws, DerivedRelationMinimumReachability,
};
pub use violation::{DerivedRelationLaw, DerivedRelationLawViolation};
