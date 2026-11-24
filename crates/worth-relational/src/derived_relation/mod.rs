//! Rebuildable endpoint sets over registered physical relation contracts.
pub(crate) mod affected;
pub(crate) mod laws;
pub use laws::{
    DerivedRelationCardinality, DerivedRelationGraphLaws, DerivedRelationLaw,
    DerivedRelationLawViolation, DerivedRelationMinimumReachability,
};
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
pub use read::{
    DerivedRelationAdjacency, DerivedRelationData, DerivedRelationReadDenial, DerivedRelationReads,
};
pub use read_shape::DerivedRelationReadShape;
pub(crate) use structural::{prepare_changes, read_changes};
pub use structural::{DerivedRelationChangeData, PreparedDerivedRelationChanges};
pub(crate) use worth_relational_contracts::internal_port::derived_relation::definition;
pub(crate) use worth_relational_contracts::internal_port::derived_relation::read;
pub(crate) use worth_relational_contracts::internal_port::derived_relation::read_denial;
pub(crate) use worth_relational_contracts::internal_port::derived_relation::read_shape;
pub(crate) mod traversal;

pub use definition::{
    DerivedRelationDefinitionParts, DerivedRelationDenial, DerivedRelationHandle,
    DerivedRelationIdentity, DerivedRelationNode,
};
