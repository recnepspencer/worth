//! The stored permutation law is judged by the ordinary native commit path.
pub use super::{creation::CreateNodeInput, installation::*, outcome::*, read};
pub use worth_query_host::facade::{
    application_invariants::{EntityReference, RelationCardinalityBoundary as Side},
    primary_graph::WorthQueryRelationLawDenial,
};

pub const CYCLE: &[(u64, u64)] = &[(0, 1), (1, 2), (2, 0)];
