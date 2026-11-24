//! Graph laws over actual canonical endpoint pairs, independent of caching.
pub(crate) mod evaluate;
mod evidence;
pub use meaning::{
    DerivedRelationCardinality, DerivedRelationGraphLaws, DerivedRelationMinimumReachability,
};
pub use violation::{DerivedRelationLaw, DerivedRelationLawViolation};
pub(crate) use worth_relational_contracts::internal_port::derived_relation::laws::meaning;
pub(crate) use worth_relational_contracts::internal_port::derived_relation::laws::violation;
