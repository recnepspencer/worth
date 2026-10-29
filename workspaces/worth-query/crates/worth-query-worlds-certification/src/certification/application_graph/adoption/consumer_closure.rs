//! Public consumer closure for branch-selected program inspection and commit.

pub use crate::facade::certification::document_retention_model::host::publish_on_first_program;
pub use crate::facade::certification::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};
pub use crate::facade::certification::document_retention_model::presented_request::set_retention;
pub use crate::facade::certification::document_retention_model::programs::RetentionProgramP1;
pub use crate::facade::certification::document_retention_model::settled_verdict::{
    settle, RetentionVerdict,
};
pub use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
pub use worth_query_host::facade::application_installation::{
    WorthQueryProgramOwner, WorthQuerySelectedProgramOwnerDenial,
};
pub use worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionDenial;
pub use worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
