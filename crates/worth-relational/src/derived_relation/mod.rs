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
mod ordered_events;
pub(crate) mod pinned;
pub(crate) mod proof;
mod source;
pub(crate) mod source_buffer;
pub(crate) mod support;
pub(crate) use pinned::read_projected_source;
pub use pinned::{
    PinnedDerivedRelationDenial, PinnedDerivedRelationRead, PinnedDerivedRelationScope,
};
pub(crate) mod structural;
pub use proof::{
    DerivedRelationReadIdentity, DerivedRelationWalkStep, PreparedDerivedRelationRead,
};
pub(crate) use structural::{prepare_changes, read_changes};
pub use structural::{DerivedRelationChangeData, PreparedDerivedRelationChanges};
pub(crate) mod read;
pub(crate) mod read_denial;
pub(crate) mod read_shape;
pub use read::{
    DerivedRelationAdjacency, DerivedRelationData, DerivedRelationReadDenial, DerivedRelationReads,
};
pub use read_shape::DerivedRelationReadShape;
pub(crate) mod definition;
pub(crate) mod traversal;
mod validation;

pub use definition::{
    DerivedRelationDefinitionParts, DerivedRelationDenial, DerivedRelationHandle,
    DerivedRelationIdentity, DerivedRelationNode,
};
