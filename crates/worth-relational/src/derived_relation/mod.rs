//! Rebuildable endpoint sets over registered physical relation contracts.
pub(crate) mod affinity;
pub(crate) mod canonical;
pub(crate) mod catalog;
mod evaluation;
mod pinned;
mod proof;
pub use pinned::PinnedDerivedRelationRead;
mod structural;
pub use proof::{DerivedRelationReadIdentity, PreparedDerivedRelationRead};
mod read;
pub use read::{
    DerivedRelationAdjacency, DerivedRelationData, DerivedRelationReadDenial, DerivedRelationReads,
};
pub(crate) mod definition;
mod validation;

pub use definition::{
    DerivedRelationDefinitionParts, DerivedRelationDenial, DerivedRelationHandle,
    DerivedRelationIdentity, DerivedRelationNode,
};
