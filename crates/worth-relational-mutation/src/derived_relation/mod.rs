//! Rebuildable endpoint sets over registered physical relation contracts.
pub mod affected;
pub mod laws;
pub use laws::{
    DerivedRelationCardinality, DerivedRelationGraphLaws, DerivedRelationLaw,
    DerivedRelationLawViolation, DerivedRelationMinimumReachability,
};
pub mod evaluation;
pub mod ordered_events;
pub mod pinned;
pub mod proof;
pub mod source;
pub mod source_buffer;
pub mod support;
pub use pinned::read_projected_source;
pub use pinned::{
    PinnedDerivedRelationDenial, PinnedDerivedRelationRead, PinnedDerivedRelationScope,
};
pub mod structural;
pub use crate::contracts::derived_relation::read::{
    DerivedRelationAdjacency, DerivedRelationData, DerivedRelationReads,
};
pub use crate::contracts::derived_relation::read_denial::DerivedRelationReadDenial;
pub use crate::contracts::derived_relation::read_shape::DerivedRelationReadShape;
pub use crate::contracts::derived_relation::{definition, read_denial};
pub use proof::{
    DerivedRelationReadIdentity, DerivedRelationWalkStep, PreparedDerivedRelationRead,
};
pub(crate) use structural::{prepare_changes, read_changes};
pub use structural::{DerivedRelationChangeData, PreparedDerivedRelationChanges};
pub mod traversal;

pub use crate::contracts::derived_relation::definition::{
    DerivedRelationDefinitionParts, DerivedRelationDenial, DerivedRelationHandle,
    DerivedRelationIdentity, DerivedRelationNode,
};
