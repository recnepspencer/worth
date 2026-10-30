pub use std::sync::atomic::Ordering;

pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::installed;

pub use super::*;
pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::{
    resource_admission_workspace, GeometryDomain, ReadExecutionInput, ReadFamily, ReadVertex,
};

pub fn bounded_request() -> installed::operation::WorthQueryExecutionResourceRequest {
    request(
        installed::operation::WorthQuerySemanticScaleRequest::bounded(1),
        installed::operation::WorthQueryResourceLimitRequest::bounded(1),
        safe_point("live-pressure-chunk"),
    )
}
