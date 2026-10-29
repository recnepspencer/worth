//! Two real canonical pair members share decode custody until atomic publication.
#[path = "paired/first_query.rs"]
pub mod first_query;
#[path = "paired/membership.rs"]
pub mod membership;
pub use super::{host, opaque, repeated_query, schema::*};
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use std::num::NonZeroUsize;
pub use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest,
};
pub use worth_foundational::facade::AspectValue;
pub use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
pub use worth_query_host::facade::declaration::application_schema::ApplicationScalarValueBinding;
pub use worth_query_host::facade::{declaration::application_query::*, primary_graph::*};

#[derive(Debug, PartialEq)]
pub struct PairRanks {
    pub first: i32,
    pub second: i32,
}
impl WorthQueryManagedDerivedValue for PairRanks {
    // Both ranks are inline; there is no retained heap payload.
    fn retained_bytes(&self) -> usize {
        0
    }
}
