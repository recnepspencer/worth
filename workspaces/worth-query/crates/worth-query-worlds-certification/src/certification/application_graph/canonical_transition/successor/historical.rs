//! Historical V1 owners remain pinned while an independent V2 installation adopts.
pub use super::super::{
    derived_query as old_query, host as old_host, output as old_output, producer as old_producer,
    program as old_program, schema as old_schema,
};
pub use super::*;
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest,
};
pub use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::{
    primary_graph::{
        WorthQueryInvariantProjectionDenialKind, WorthQueryProductBranchAdmissionDenial,
        WorthQueryProductBranchAdmissionRefusalKind,
    },
    runtime::ExecutionAllocationPolicy,
};

pub fn assert_v1_literals(value: &old_output::DerivedValue) {
    assert_eq!(value.caption, "ASSEMBLY Β");
    assert_eq!(value.rank_sum, 12);
    assert_eq!(value.tags, ["left", "right"]);
}
