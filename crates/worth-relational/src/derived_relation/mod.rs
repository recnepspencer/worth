//! Rebuildable endpoint sets over registered physical relation contracts.
pub(crate) mod affected;
pub(crate) mod laws;
pub use laws::{
    DerivedRelationCardinality, DerivedRelationGraphLaws, DerivedRelationLaw,
    DerivedRelationLawViolation, DerivedRelationMinimumReachability,
};
pub(crate) mod canonical;
pub(crate) mod catalog;
pub(crate) mod evaluation;
mod pinned;
mod proof;
mod source;
pub use pinned::PinnedDerivedRelationRead;
mod structural;
pub use proof::{
    DerivedRelationReadIdentity, DerivedRelationReadShape, DerivedRelationWalkStep,
    PreparedDerivedRelationRead,
};
pub(crate) use structural::{prepare_changes, read_changes};
pub use structural::{DerivedRelationChangeData, PreparedDerivedRelationChanges};
mod read;
pub use read::{
    DerivedRelationAdjacency, DerivedRelationData, DerivedRelationReadDenial, DerivedRelationReads,
};
pub(crate) mod definition;
pub(crate) mod traversal;
mod validation;

pub use definition::{
    DerivedRelationDefinitionParts, DerivedRelationDenial, DerivedRelationHandle,
    DerivedRelationIdentity, DerivedRelationNode,
};
