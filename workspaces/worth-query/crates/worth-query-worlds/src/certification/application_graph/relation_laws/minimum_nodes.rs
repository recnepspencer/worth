//! Distinct reachability through cycles and repeated physical paths.
pub use super::{creation::CreateNodeInput, installation::*, outcome::*, read, schema::*};
pub use worth_query_host::facade::{
    application_invariants::EntityReference, primary_graph::WorthQueryRelationLawDenial,
};
pub const CYCLE: &[(u64, u64)] = &[(0, 1), (1, 2), (2, 0)];
