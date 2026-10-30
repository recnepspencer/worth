//! Ordinary precise retirement preserves already owned historical results.
pub use super::*;
pub use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use worth_query_host::facade::{application_entry::*, runtime::ExecutionAllocationPolicy};
